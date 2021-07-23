use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};
use std::sync::{Arc, Mutex};
pub struct Mset;

impl Runnable<Arc<Mutex<Database>>> for Mset {
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

        let keys_and_value = buffer.chunks(2);
        keys_and_value.into_iter().for_each(|pair_key_value| {
            database.insert(
                pair_key_value[0].to_string(),
                TypeSaved::String(pair_key_value[1].to_string()),
            );
        });

        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    check_empty(buffer, "mset")?;

    if buffer.is_empty() || is_odd(buffer) {
        // never odd => "key1 value1 key2 value2 ...""
        let error_message = redis_messages::arguments_invalid_to("mset");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

fn is_odd(buffer: &[String]) -> bool {
    buffer.len() % 2 == 1
}

#[cfg(test)]
mod test_mset_function {
    use crate::commands::create_notifier;
    use crate::{native_types::RBulkString, vec_strings};

    use super::*;

    #[test]
    fn test_01_mset_reemplace_value_old_of_key_and_insert_more_elements() {
        let buffer_mock2 = vec_strings!["key1", "value1_new", "key2", "value2"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key1".to_string(), TypeSaved::String("value1".to_string()));

        let _ = Mset.run(buffer_mock2, &mut database_mock);

        let mut get_received_1 = String::new();
        if let TypeSaved::String(item) = database_mock.lock().unwrap().get("key1").unwrap() {
            get_received_1 = RBulkString::encode(item.to_string());
        }
        let expected = RBulkString::encode("value1_new".to_string());
        assert_eq!(expected, get_received_1);

        let mut get_received_2 = String::new();
        if let TypeSaved::String(item) = database_mock.lock().unwrap().get("key2").unwrap() {
            get_received_2 = RBulkString::encode(item.to_string());
        }

        let expected = RBulkString::encode("value2".to_string());
        assert_eq!(expected, get_received_2);
    }

    #[test]
    fn test_02_mset_with_bad_args_return_err() {
        let buffer_mock = vec_strings!["key1", "value1_new", "key2"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        let result_received = Mset.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("mset");
        let expected_result: String =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
