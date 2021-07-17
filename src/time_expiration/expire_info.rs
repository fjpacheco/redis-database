use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use crate::{communication::log_messages::LogMessage, tcp_protocol::notifier::Notifier};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub struct ExpireInfo {
    last_touch: SystemTime,
    timeout: Option<Duration>,
}
impl Default for ExpireInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpireInfo {
    pub fn new() -> ExpireInfo {
        ExpireInfo {
            last_touch: SystemTime::now(),
            timeout: None,
        }
    }

    pub fn is_expired(&mut self, notifier: Option<Arc<Mutex<Notifier>>>, key_name: &str) -> bool {
        if self.timeout.is_some() {
            let _ = self.update(notifier, key_name);
            !matches!(self.timeout, Some(_))
        } else {
            false
        }
    }

    pub fn update(
        &mut self,
        wrapped_notifier: Option<Arc<Mutex<Notifier>>>,
        key_name: &str,
    ) -> Result<(), ErrorStruct> {
        let previous_touch = self.last_touch;
        self.last_touch = SystemTime::now();
        if let Some(ttl) = self.timeout {
            let difference = duration_since(&self.last_touch, previous_touch)?;
            //let difference =  self.last_touch.duration_since(previous_touch).unwrap();
            self.timeout = ttl.checked_sub(difference);
        }
        if let Some(notifier) = wrapped_notifier {
            let from_epoch = duration_since(&previous_touch, UNIX_EPOCH)?;
            notifier
                .lock()
                .map_err(|_| {
                    ErrorStruct::from(redis_messages::poisoned_lock(
                        "Notifier",
                        ErrorSeverity::ShutdownServer,
                    ))
                })?
                .send_log(LogMessage::key_touched(key_name, from_epoch.as_secs()))?;
        }
        Ok(())
    }

    pub fn ttl(&self) -> Option<u64> {
        self.timeout.map(|ttl| ttl.as_secs())
    }

    pub fn set_timeout(&mut self, duration: u64) -> Result<(), ErrorStruct> {
        self.last_touch = SystemTime::now();
        self.timeout = Some(Duration::new(duration, 0));
        Ok(())
    }

    pub fn set_timeout_unix_timestamp(&mut self, duration: u64) -> Result<(), ErrorStruct> {
        self.last_touch = SystemTime::now();
        duration_since(&self.last_touch, UNIX_EPOCH).and_then(|duration_since_epoch| {
            self.timeout = Some(Duration::new(duration, 0) - duration_since_epoch);
            Ok(())
        })
    }

    pub fn persist(&mut self) -> Option<u64> {
        self.timeout.take().map(|ttl| ttl.as_secs())
    }
}

// DESHABILITENLO PARA NO COMERSE PRUEBAS QUE DURAN BANDA (O SEA 5 PRECIOSOS SEGUNDOS)

fn duration_since(time: &SystemTime, previous_time: SystemTime) -> Result<Duration, ErrorStruct> {
    time.duration_since(previous_time)
        .map_err(|_| ErrorStruct::from(redis_messages::ttl_epoch_error()))
}

#[cfg(test)]
mod test_clock {

    use super::*;
    use std::thread::sleep;

    #[test]
    #[ignore]
    fn test01_new_expireinfo_does_not_have_timeout() {
        let mut info = ExpireInfo::new();
        assert!(!info.is_expired(None, "key"));
        assert_eq!(info.ttl(), None);
    }

    #[test]
    #[ignore]
    fn test02_setting_ten_seconds_of_timeout() {
        let mut info = ExpireInfo::new();
        info.set_timeout(10).unwrap();
        assert_eq!(info.ttl(), Some(10));
        sleep(Duration::new(5, 0));
        assert!(!info.is_expired(None, "key"));
        assert_eq!(info.ttl(), Some(4));
        sleep(Duration::new(6, 0));
        assert!(info.is_expired(None, "key"));
        assert_eq!(info.ttl(), None);
    }

    #[test]
    #[ignore]
    fn test03_setting_five_seconds_of_timeout_with_unix_timestamp() {
        let mut info = ExpireInfo::new();
        match SystemTime::now().duration_since(UNIX_EPOCH - Duration::new(5, 0)) {
            Ok(dura) => info.set_timeout_unix_timestamp(dura.as_secs()).unwrap(),
            Err(_) => {}
        }
        assert_eq!(info.ttl(), Some(4));
        sleep(Duration::new(2, 0));
        assert!(info.update(None, "key").is_ok());
        assert_eq!(info.ttl(), Some(2));
    }
}
