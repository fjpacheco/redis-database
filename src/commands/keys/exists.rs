use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, check_not_empty, Runnable},
    database::Database,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};
use std::sync::{Arc, Mutex};

pub struct Exists;

impl Runnable<Arc<Mutex<Database>>> for Exists {
    /// Returns if key exists.
    ///
    /// # Return value
    /// * [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): 1 if the timeout was set.
    /// * [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): 0 if key does not exist.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty, or received with a number of elements
    /// different than 1.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        mut buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        check_empty(&buffer, "exists")?;
        let key = buffer.pop().unwrap();
        check_not_empty(&buffer)?;
        if database.contains_key(&key) {
            Ok(RInteger::encode(1))
        } else {
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
mod test_exists {
    use crate::commands::create_notifier;

    use super::*;
    use crate::database::TypeSaved;

    #[test]
    fn test_01_exists_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock = vec!["key".to_string()];
        let result_received = Exists.run(buffer_mock, &mut database);
        assert_eq!(RInteger::encode(1), result_received.unwrap());
    }

    #[test]
    fn test_02_exists_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock = vec!["key1".to_string()];
        let result_received = Exists.run(buffer_mock, &mut database);
        assert_eq!(RInteger::encode(0), result_received.unwrap());
    }
}
