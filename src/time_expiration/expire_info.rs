use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use crate::{communication::log_messages::LogMessage, tcp_protocol::notifier::Notifier};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Clone)]
/// This structure contains information about the
/// time to live of a key. It has the instant of
/// the last access to the key, and the value of
/// the timeout of the key (if there is one).
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
    // Creates the structure
    pub fn new() -> ExpireInfo {
        ExpireInfo {
            last_touch: SystemTime::now(),
            timeout: None,
        }
    }

    /// Evaluates if the timeout has ended.
    #[allow(clippy::branches_sharing_code)]
    pub fn is_expired(&mut self, notifier: Option<Arc<Mutex<Notifier>>>, key_name: &str) -> bool {
        if self.timeout.is_some() {
            let _ = self.update(notifier, key_name);
            !matches!(self.timeout, Some(_))
        } else {
            let _ = self.update(notifier, key_name);
            false
        }
    }

    /// Updates the last access to the info and
    /// the remaining time to live.
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

    /// Returns the timeout as seconds.
    pub fn ttl(&self) -> Option<u64> {
        self.timeout.map(|ttl| ttl.as_secs())
    }

    /// Sets a new timeout for the structure from seconds.
    pub fn set_timeout(&mut self, duration: u64) -> Result<(), ErrorStruct> {
        self.last_touch = SystemTime::now();
        self.timeout = Some(Duration::new(duration, 0));
        Ok(())
    }

    /// Sets a new timeout for the structure from a time coded
    /// in Unix timestamp.
    pub fn set_timeout_unix_timestamp(&mut self, duration: u64) -> Result<(), ErrorStruct> {
        self.last_touch = SystemTime::now();

        duration_since(&self.last_touch, UNIX_EPOCH).map(|duration_since_epoch| {
            self.timeout = Some(Duration::new(duration, 0) - duration_since_epoch);
        })
    }

    /// Takes the timeout of the structure and returns it.
    pub fn persist(&mut self) -> Option<u64> {
        self.timeout.take().map(|ttl| ttl.as_secs())
    }
}

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
