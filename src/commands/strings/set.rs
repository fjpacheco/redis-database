use std::sync::{Arc, Mutex};

use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    messages::redis_messages,
    native_types::{error_severity::ErrorSeverity, ErrorStruct, RSimpleString, RedisType},
};

pub struct Set;

impl Runnable<Arc<Mutex<Database>>> for Set {
    /// Set key to hold the string value. If key already holds a value, it is overwritten, regardless of its type.
    /// Any previous time to live associated with the key is discarded on successful SET operation.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if SET was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The buffer [Vec]<[String]> more than two elements is received or empty.
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
        check_error_cases(&buffer)?;

        let value = buffer[1].to_string();
        let key = buffer[0].to_string();

        database.insert(key, TypeSaved::String(value)); // replace any old value with this key
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    check_empty(buffer, "set")?;

    if buffer.len() == 1 {
        // never "arg1"
        let error_message = redis_messages::arguments_invalid_to("set");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    // Different error output => checked with src/redis-server!!
    if buffer.len() != 2 {
        // never "arg1 arg2 arg3 ... "
        let error_message = redis_messages::syntax_error();
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_set_function {
    use crate::commands::create_notifier;

    use crate::{native_types::RBulkString, vec_strings};

    use super::*;
    #[test]
    fn test_01_set_key_and_value_return_ok() {
        let buffer_mock = vec_strings!["key", "value"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        let result_received = Set.run(buffer_mock, &mut database_mock);

        let expected_result: String = ("+".to_owned() + &redis_messages::ok() + "\r\n").to_string();
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test_02_set_key_and_value_save_correctly_in_database_mock() {
        let buffer_mock_set = vec_strings!["key", "value"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        let _ = Set.run(buffer_mock_set, &mut database_mock);
        let mut get_received = String::new();
        if let TypeSaved::String(item) = database_mock.lock().unwrap().get("key").unwrap() {
            get_received = RBulkString::encode(item.to_string());
        }

        let expected = RBulkString::encode("value".to_string());
        assert_eq!(expected, get_received);
    }

    #[test]
    fn test_03_set_key_and_value_but_get_another_key_return_none() {
        let buffer_mock = vec_strings!["key", "value"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        let _ = Set.run(buffer_mock, &mut database_mock);
        let mut get_received = String::new();
        if let TypeSaved::String(item) = database_mock
            .lock()
            .unwrap()
            .get("key2")
            .unwrap_or(&TypeSaved::String("(nil)".to_string()))
        {
            get_received = RBulkString::encode(item.to_string());
        }

        let expected = RBulkString::encode("(nil)".to_string());
        assert_eq!(expected, get_received);
    }

    #[test]
    fn test_04_set_without_value_and_key_return_err_syntax() {
        let buffer_mock = vec_strings!["set", "set", "set", "set"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        let result_received = Set.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::syntax_error();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
