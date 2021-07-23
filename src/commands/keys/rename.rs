use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty_2, check_not_empty, Runnable},
    database::Database,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};
use std::sync::{Arc, Mutex};
pub struct Rename;

/// Renames key to newkey. It returns an error when key does not exist.
/// If newkey already exists it is overwritten, when this happens RENAME
/// executes an implicit DEL operation, so if the deleted key contains a
/// very big value it may cause high latency even if RENAME itself is
/// usually a constant-time operation.

impl Runnable<Arc<Mutex<Database>>> for Rename {
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
        check_not_empty(&buffer)?;
        let new_key = buffer.pop().unwrap();
        check_not_empty(&buffer)?;
        let old_key = buffer.pop().unwrap();
        check_empty_2(&buffer)?;
        if let Some(string_list) = database.get(&old_key) {
            let value = string_list.clone();
            database.remove(&old_key);
            database.insert(new_key, value);
            Ok(RSimpleString::encode("OK".to_string()))
        } else {
            Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("no such key"),
            ))
        }
    }
}

#[cfg(test)]
mod test_rename {
    use crate::commands::create_notifier;

    use super::*;
    use crate::{
        commands::strings::get::Get, database::TypeSaved, native_types::RBulkString, vec_strings,
    };

    #[test]
    fn test_01_rename_existing_key_with_new_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_1 = vec_strings!["key", "new_key"];
        let result1 = Rename.run(buffer_mock_1, &mut database);
        assert_eq!(result1.unwrap(), "+OK\r\n".to_string());
        let buffer_mock_2 = vec_strings!["new_key"];
        let result2 = Get.run(buffer_mock_2, &mut database);
        assert_eq!(RBulkString::encode("value".to_string()), result2.unwrap());
    }

    #[test]
    fn test_02_rename_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock = vec_strings!["random_key", "new_key"];
        let error = Rename.run(buffer_mock, &mut database);
        assert_eq!(error.unwrap_err().print_it(), "ERR no such key".to_string());
    }

    #[test]
    fn test_03_rename_existing_key_with_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_1 = vec_strings!["key", "key"];
        let result1 = Rename.run(buffer_mock_1, &mut database);
        assert_eq!(result1.unwrap(), "+OK\r\n".to_string());
        let buffer_mock_2 = vec_strings!["key"];
        let result2 = Get.run(buffer_mock_2, &mut database);
        assert_eq!(RBulkString::encode("value".to_string()), result2.unwrap());
    }
}
