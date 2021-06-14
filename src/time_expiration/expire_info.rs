use std::time::Duration;
use std::time::SystemTime;
//use std::time::Instant;
use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use std::time::UNIX_EPOCH;

pub struct ExpireInfo {
    last_touch: SystemTime,
    timeout: Option<Duration>,
}

impl ExpireInfo {
    pub fn new() -> ExpireInfo {
        ExpireInfo {
            last_touch: SystemTime::now(),
            timeout: None,
        }
    }

    pub fn is_expired(&mut self) -> bool {
        if let Some(_) = self.timeout {
            self.update();

            if let Some(_) = self.timeout {
                false
            } else {
                true
            }
        } else {
            self.update();
            false
        }
    }

    pub fn update(&mut self) {
        let previous_touch = self.last_touch;
        self.last_touch = SystemTime::now();
        let difference = self.last_touch.duration_since(previous_touch).unwrap();
        if let Some(ttl) = self.timeout {
            self.timeout = ttl.checked_sub(difference);
        }
    }

    pub fn ttl(&self) -> Option<u64> {
        //No es necesario hacer update()
        //self.update();
        if let Some(ttl) = self.timeout {
            Some(ttl.as_secs())
        } else {
            None
        }
    }

    pub fn set_timeout(&mut self, duration: u64) -> Result<(), ErrorStruct> {
        self.last_touch = SystemTime::now();
        self.timeout = Some(Duration::new(duration, 0));
        Ok(())
    }

    pub fn set_timeout_unix_timestamp(&mut self, duration: u64) -> Result<(), ErrorStruct> {
        self.last_touch = SystemTime::now();
        match self.last_touch.duration_since(UNIX_EPOCH) {
            Ok(duration_since_epoch) => {
                self.timeout = Some(Duration::new(duration, 0) - duration_since_epoch);
                Ok(())
            }
            Err(_) => {
                let message = redis_messages::ttl_error();
                Err(ErrorStruct::new(
                    message.get_prefix(),
                    message.get_message(),
                ))
            }
        }
    }

    pub fn persist(&mut self) -> Option<u64> {
        if let Some(ttl) = self.timeout {
            self.timeout = None;
            Some(ttl.as_secs())
        } else {
            None
        }
    }
}

// DESHABILITENLO PARA NO COMERSE PRUEBAS QUE DURAN BANDA (O SEA 5 PRECIOSOS SEGUNDOS)

#[cfg(test)]
mod test_clock {

    use super::*;
    use std::thread::sleep;

    #[test]
    #[ignore]
    fn test01_new_expireinfo_does_not_have_timeout() {
        let mut info = ExpireInfo::new();
        assert!(!info.is_expired());
        assert_eq!(info.ttl(), None);
    }

    #[test]
    #[ignore]
    fn test02_setting_ten_seconds_of_timeout() {
        let mut info = ExpireInfo::new();
        info.set_timeout(10).unwrap();
        assert_eq!(info.ttl(), Some(10));
        sleep(Duration::new(5, 0));
        assert!(!info.is_expired());
        assert_eq!(info.ttl(), Some(4));
        sleep(Duration::new(6, 0));
        assert!(info.is_expired());
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
        info.update();
        assert_eq!(info.ttl(), Some(2));
    }
}
