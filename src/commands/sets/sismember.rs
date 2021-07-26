use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};
use std::sync::{Arc, Mutex};
pub struct Sismember;

impl Runnable<Arc<Mutex<Database>>> for Sismember {
    /// Returns if member is a member of the set stored at key.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): specifically:
    /// * 1 if the element is a member of the set.
    /// * 0 if the element is not a member of the set, or if **key** does not exist.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a set.
    /// * Buffer [Vec]<[String]> is received empty, or not received with only two element.
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
                TypeSaved::Set(item) => {
                    let member = &buffer[1];
                    let result = match item.contains(member) {
                        true => 1,
                        false => 0,
                    };
                    Ok(RInteger::encode(result))
                }
                _ => {
                    err_wrongtype!()
                }
            },
            None => Ok(RInteger::encode(0)),
        }
    }
}

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    check_empty(&buffer, "sismember")?;

    if buffer.len() != 2 {
        let error_message = redis_messages::arguments_invalid_to("sismember");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod test_sismember_function {
    use crate::commands::create_notifier;
    use std::collections::{HashSet, VecDeque};

    use crate::vec_strings;

    use super::*;

    #[test]
    fn test_01_sismember_return_number_one_if_member_is_contained_in_set() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock_1 = vec_strings!["key", "m1"];
        let buffer_mock_2 = vec_strings!["key", "m2"];

        let result_received_1 = Sismember.run(buffer_mock_1, &mut database_mock);
        let result_received_2 = Sismember.run(buffer_mock_2, &mut database_mock);

        let excepted_1 = RInteger::encode(1);
        let excepted_2 = RInteger::encode(1);
        assert_eq!(excepted_1, result_received_1.unwrap());
        assert_eq!(excepted_2, result_received_2.unwrap());
    }

    #[test]
    fn test_02_sismember_return_number_zero_if_member_is_not_contained_in_set() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key", "m_random"];

        let result_received = Sismember.run(buffer_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test_03_sismember_return_number_zero_if_the_key_of_set_dont_exist_in_database() {
        // TODO: revisar nombres de tests... "in database" ...
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key_random", "m_random"];

        let result_received = Sismember.run(buffer_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test_04_sismember_return_error_wrongtype_if_execute_with_key_of_string() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock.lock().unwrap().insert(
            "keyOfString".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_mock = vec_strings!["keyOfString", "value"];

        let result_received = Sismember.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test_05_sismember_return_error_wrongtype_if_execute_with_key_of_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());
        new_list.push_back("value_other".to_string());
        database_mock
            .lock()
            .unwrap()
            .insert("keyOfList".to_string(), TypeSaved::List(new_list));

        let buffer_mock = vec_strings!["keyOfList", "value"];

        let result_received = Sismember.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test_06_sismember_return_error_arguments_invalid_ifbuffer_has_more_than_3_arguments() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer_mock = vec_strings!["arg1", "arg2", "arg3"];

        let result_received = Sismember.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("sismember");
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
