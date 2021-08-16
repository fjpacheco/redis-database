use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RedisType},
};
use std::sync::{Arc, Mutex};
pub struct Smembers;

impl Runnable<Arc<Mutex<Database>>> for Smembers {
    /// Returns all the members of the set value stored at **key**.
    ///
    /// # Return value
    /// [String] _encoded_ in [RArray]: all elements of the set.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a set.
    /// * Buffer [Vec]<[String]> is received empty, or not received with only one element.
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

        let key = &buffer[0];

        match database.get_mut(key) {
            Some(item) => match item {
                TypeSaved::Set(a_set) => {
                    let vector: Vec<String> =
                        a_set.iter().map(|member| member.to_string()).collect();
                    Ok(RArray::encode(vector))
                }
                _ => {
                    err_wrongtype!()
                }
            },
            None => Ok(RArray::encode(vec![])), // Empty array! => "*0\r\n"
        }
    }
}

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    check_empty(buffer, "smembers")?;

    if buffer.len() != 1 {
        let error_message = redis_messages::arguments_invalid_to("smembers");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_smembers_function {
    use crate::commands::create_notifier;
    use std::collections::{HashSet, VecDeque};

    use crate::vec_strings;

    use super::*;

    #[test]
    fn test_01_smembers_return_array_members_of_set_not_necessarily_ordered() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key"];

        let result_received = Smembers.run(buffer_mock, &mut database_mock);
        let array = result_received.unwrap();

        // The Redis Sets are not necessarily ordered. That is why it is analyzed in lower level at Array Native Type.
        assert!(array.contains("*2\r\n"));
        assert!(array.contains("$2\r\nm1\r\n"));
        assert!(array.contains("$2\r\nm2\r\n"));
    }

    #[test]
    fn test_02_smembers_return_an_empty_array_if_key_does_not_exist_in_database() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key_other"];

        let result_received = Smembers.run(buffer_mock, &mut database_mock);

        let excepted = RArray::encode(vec![]); // Empty array! => "*0\r\n"
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test_03_smembers_return_an_empty_array_if_set_is_empty() {
        let set = HashSet::new();
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key"];

        let result_received = Smembers.run(buffer_mock, &mut database_mock);

        let excepted = RArray::encode(vec![]); // Empty array! => "*0\r\n"
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test_04_smembers_return_error_wrongtype_if_execute_with_key_of_string() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock.lock().unwrap().insert(
            "keyOfString".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_mock = vec_strings!["keyOfString"];

        let result_received = Smembers.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test_05_smembers_return_error_wrongtype_if_execute_with_key_of_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        database_mock
            .lock()
            .unwrap()
            .insert("keyOfList".to_string(), TypeSaved::List(new_list));

        let buffer_mock = vec_strings!["keyOfList"];

        let result_received = Smembers.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test_06_smembers_return_error_arguments_invalid_ifbuffer_has_many_one_arguments() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer_mock = vec_strings!["arg1", "arg2", "arg3"];

        let result_received = Smembers.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("smembers");
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
