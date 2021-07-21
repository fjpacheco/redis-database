use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, Runnable},
    database::Database,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};
use std::sync::{Arc, Mutex};

pub struct Copy;

impl Runnable<Arc<Mutex<Database>>> for Copy {
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
        check_error_cases(&buffer)?;

        let key_source = &buffer[0];
        let key_destinty = &buffer[1];

        if !database.contains_key(&key_source) | database.contains_key(&key_destinty) {
            Ok(RInteger::encode(0))
        } else {
            let value = database.get(&key_source).unwrap().clone(); // Unwrap Reason: Value associated for Key Source exists!
            database.insert(key_destinty.to_string(), value);
            Ok(RInteger::encode(1))
        }
    }
}

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    check_empty(&buffer, "copy")?;

    if buffer.len() != 2 {
        // never "copy" or "copy arg1"
        let error_message = redis_messages::arguments_invalid_to("copy");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_copy_function {
    use crate::commands::create_notifier;

    use std::collections::{HashSet, VecDeque};

    use crate::{database::TypeSaved, vec_strings};

    use super::*;

    #[test]
    fn test01_copy_value_string_of_key_source_existent_into_key_destiny_non_existent_return_success_one(
    ) {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_get = vec_strings!["key", "key_new"];

        let result_received = Copy.run(buffer_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(1);
        assert_eq!(expected_result, result_received.unwrap());

        let mut mutex_db = database_mock.lock().unwrap();
        if let TypeSaved::String(set_post_copy) = mutex_db.get("key").unwrap() {
            assert!(set_post_copy.contains("value"));
        }

        if let TypeSaved::String(set_post_copy) = mutex_db.get("key_new").unwrap() {
            assert!(set_post_copy.contains("value"));
        }
    }

    #[test]
    fn test02_copy_value_string_of_key_source_existent_into_key_destiny_existent_return_error_zero()
    {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        database_mock.lock().unwrap().insert(
            "key_new".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_mock_get = vec_strings!["key", "key_new"];

        let result_received = Copy.run(buffer_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(0);
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test03_copy_value_string_of_key_source_non_existent_into_key_destiny_non_existent_return_error_zero(
    ) {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_get = vec_strings!["key_random", "key_new"];

        let result_received = Copy.run(buffer_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(0);
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test04_copy_value_set_of_key_source_existent_into_key_destiny_non_existent_return_success_one(
    ) {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));

        let buffer_mock_get = vec_strings!["key", "key_new"];

        let result_received = Copy.run(buffer_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(1);
        assert_eq!(expected_result, result_received.unwrap());
        let mut mutex_db = database_mock.lock().unwrap();

        if let TypeSaved::Set(set_post_copy) = mutex_db.get("key").unwrap() {
            assert!(set_post_copy.contains("m1"));
            assert!(set_post_copy.contains("m2"));
            assert!(set_post_copy.len().eq(&2))
        }

        if let TypeSaved::Set(set_post_copy) = mutex_db.get("key_new").unwrap() {
            assert!(set_post_copy.contains("m1"));
            assert!(set_post_copy.contains("m2"));
            assert!(set_post_copy.len().eq(&2))
        }
    }

    #[test]
    fn test06_copy_value_list_of_key_source_existent_into_key_destiny_non_existent_return_success_one(
    ) {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer_mock_get = vec_strings!["key", "key_new"];

        let result_received = Copy.run(buffer_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(1);
        assert_eq!(expected_result, result_received.unwrap());
        let mut mutex_db = database_mock.lock().unwrap();

        if let TypeSaved::List(set_post_copy) = mutex_db.get("key").unwrap() {
            assert!(set_post_copy.contains(&"value1".to_string()));
            assert!(set_post_copy.contains(&"value2".to_string()));
            assert!(set_post_copy.len().eq(&2))
        }

        if let TypeSaved::List(set_post_copy) = mutex_db.get("key_new").unwrap() {
            assert!(set_post_copy.contains(&"value1".to_string()));
            assert!(set_post_copy.contains(&"value2".to_string()));
            assert!(set_post_copy.len().eq(&2))
        }
    }
}
