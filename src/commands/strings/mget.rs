use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RedisType},
};
use std::sync::{Arc, Mutex};
pub struct Mget;

impl Runnable<Arc<Mutex<Database>>> for Mget {
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

        let mut values_obtained: Vec<String> = Vec::new();
        buffer
            .iter()
            .for_each(|key| match database.get(&key.to_string()) {
                Some(value) => match value {
                    TypeSaved::String(value) => values_obtained.push(value.to_string()),
                    _ => values_obtained.push("(nil)".to_string()),
                },
                None => {
                    values_obtained.push("(nil)".to_string());
                }
            });
        Ok(RArray::encode(values_obtained))
    }
}

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    check_empty(buffer, "mget")?;

    if buffer.len() == 1 {
        // never "mget" alone
        let error_message = redis_messages::arguments_invalid_to("mget");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_get {
    use crate::commands::create_notifier;

    use crate::vec_strings;

    use super::*;

    #[test]
    fn test_01_mget_value_of_key_correct_is_success() {
        let buffer_mock_get = vec_strings!["key2", "asd", "key1"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        database_mock
            .lock()
            .unwrap()
            .insert("key1".to_string(), TypeSaved::String("value1".to_string()));
        database_mock
            .lock()
            .unwrap()
            .insert("key2".to_string(), TypeSaved::String("value2".to_string()));
        let result_received = Mget.run(buffer_mock_get, &mut database_mock);

        // ->> "*3\r\n $5\r\nvalue\r\n $-1\r\n $5\r\nvalue\r\n"
        let expected_vec = vec![
            "value2".to_string(),
            "(nil)".to_string(),
            "value1".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());
    }

    #[test]
    fn test_02_mget_does_not_maintain_order() {
        let buffer_mock_get1 = vec_strings!["key2", "asd", "key1"];
        let buffer_mock_get2 = vec_strings!["asd", "key2", "key1"];
        let buffer_mock_get3 = vec_strings!["key1", "key2", "asd"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        database_mock
            .lock()
            .unwrap()
            .insert("key1".to_string(), TypeSaved::String("value1".to_string()));
        database_mock
            .lock()
            .unwrap()
            .insert("key2".to_string(), TypeSaved::String("value2".to_string()));

        let result_received = Mget.run(buffer_mock_get1, &mut database_mock);
        let expected_vec = vec![
            "value2".to_string(),
            "(nil)".to_string(),
            "value1".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());

        let result_received = Mget.run(buffer_mock_get2, &mut database_mock);
        let expected_vec = vec![
            "(nil)".to_string(),
            "value2".to_string(),
            "value1".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());

        let result_received = Mget.run(buffer_mock_get3, &mut database_mock);
        let expected_vec = vec![
            "value1".to_string(),
            "value2".to_string(),
            "(nil)".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());
    }
}
