use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, check_not_empty, Runnable},
    database::Database,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};
use std::sync::{Arc, Mutex};

pub struct Del;

impl Runnable<Arc<Mutex<Database>>> for Del {
    /// Removes the specified keys. A key is ignored if it does not exist.
    /// Time complexity: O(N) where N is the number of keys that will be
    /// removed. When a key to remove holds a value other than a string,
    /// the individual complexity for this key is O(M) where M is the number
    /// of elements in the list, set, sorted set or hash. Removing a single
    /// key that holds a string value is O(1).
    ///
    /// # Return value
    /// * [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): The number of keys that were removed.
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
        check_empty(&buffer, "del")?;
        let key = buffer.pop().unwrap();
        check_not_empty(&buffer)?;
        if database.remove(&key).is_some() {
            Ok(RInteger::encode(1))
        } else {
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
mod test_del {
    use crate::commands::create_notifier;

    use super::*;
    use crate::{database::TypeSaved, vec_strings};

    #[test]
    fn test_01_del_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_del = vec_strings!["key"];
        let result_received = Del.run(buffer_mock_del, &mut database);
        assert_eq!(RInteger::encode(1), result_received.unwrap());
    }

    #[test]
    fn test_02_del_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_del = vec_strings!["key1"];
        let result_received = Del.run(buffer_mock_del, &mut database);
        assert_eq!(RInteger::encode(0), result_received.unwrap());
    }

    #[test]
    fn test_01_del_key_just_deleted() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_del_1 = vec_strings!["key"];
        let result1 = Del.run(buffer_mock_del_1, &mut database);
        assert_eq!(RInteger::encode(1), result1.unwrap());
        let buffer_mock_del_2 = vec_strings!["key"];
        let result2 = Del.run(buffer_mock_del_2, &mut database);
        assert_eq!(RInteger::encode(0), result2.unwrap());
    }
}
