use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RInteger, RedisType},
    Database,
};
use std::sync::{Arc, Mutex};
pub struct Touch;

impl Runnable<Arc<Mutex<Database>>> for Touch {
    /// Alters the last access time of a key(s). A key is ignored if it does not exist.
    /// Time complexity: O(N) where N is the number of keys that will be touched.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger]: The number of keys that were touched.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        Ok(RInteger::encode(
            buffer
                .iter()
                .map(|key| if database.contains_key(key) { 1 } else { 0 })
                .sum(),
        ))
    }
}

#[cfg(test)]
pub mod test_touch {
    use super::*;
    use crate::commands::create_notifier;
    use crate::database::TypeSaved;

    #[test]
    fn test_01_three_keys_touch_three() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut db = Database::new(notifier);

        db.insert("key1".to_string(), TypeSaved::String("a".to_string()));
        db.insert("key2".to_string(), TypeSaved::String("b".to_string()));
        db.insert("key3".to_string(), TypeSaved::String("c".to_string()));
        let mut c_db = Arc::new(Mutex::new(db));

        let sum = Touch.run(
            vec!["key1".to_string(), "key2".to_string(), "key3".to_string()],
            &mut c_db,
        );

        assert_eq!(&sum.unwrap(), ":3\r\n");
    }

    #[test]
    fn test_02_three_keys_touch_two() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut db = Database::new(notifier);

        db.insert("key1".to_string(), TypeSaved::String("a".to_string()));
        db.insert("key2".to_string(), TypeSaved::String("b".to_string()));
        db.insert("key3".to_string(), TypeSaved::String("c".to_string()));

        let mut c_db = Arc::new(Mutex::new(db));

        let sum = Touch.run(vec!["key1".to_string(), "key3".to_string()], &mut c_db);

        assert_eq!(&sum.unwrap(), ":2\r\n");
    }

    #[test]
    fn test_03_three_keys_touch_four() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut db = Database::new(notifier);

        db.insert("key1".to_string(), TypeSaved::String("a".to_string()));
        db.insert("key2".to_string(), TypeSaved::String("b".to_string()));
        db.insert("key3".to_string(), TypeSaved::String("c".to_string()));

        let mut c_db = Arc::new(Mutex::new(db));
        let sum = Touch.run(
            vec![
                "key1".to_string(),
                "key2".to_string(),
                "key3".to_string(),
                "key4".to_string(),
            ],
            &mut c_db,
        );

        assert_eq!(&sum.unwrap(), ":3\r\n");
    }

    #[test]
    fn test_04_three_keys_touch_zero() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut db = Database::new(notifier);

        db.insert("key1".to_string(), TypeSaved::String("a".to_string()));
        db.insert("key2".to_string(), TypeSaved::String("b".to_string()));
        db.insert("key3".to_string(), TypeSaved::String("c".to_string()));
        let mut c_db = Arc::new(Mutex::new(db));
        let sum = Touch.run(vec![], &mut c_db);

        assert_eq!(&sum.unwrap(), ":0\r\n");
    }
}
