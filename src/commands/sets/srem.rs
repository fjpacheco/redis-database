use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};
use std::sync::{Arc, Mutex};
pub struct Srem;

impl Runnable<Arc<Mutex<Database>>> for Srem {
    /// Remove the specified members from the set stored at key. Specified members that are not a member of this set are ignored.
    /// If key does not exist, it is treated as an empty set and this command returns 0.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): the number of members that were removed from the set, not including non existing members.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a set.
    /// * Buffer [Vec]<[String]> is received empty, or received with only one element.
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
                    let count_deleted = buffer
                        .iter()
                        .skip(1)
                        .map(|member| item.remove(member))
                        .filter(|x| *x)
                        .count();

                    Ok(RInteger::encode(count_deleted as isize))
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
    check_empty(&buffer, "srem")?;

    if buffer.len() < 2 {
        let error_message = redis_messages::arguments_invalid_to("srem");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_srem_function {
    use crate::commands::create_notifier;
    use std::collections::{HashSet, VecDeque};

    use crate::vec_strings;

    use super::*;

    #[test]
    fn test_01_srem_remove_members_of_set_and_return_the_eliminated_amount() {
        let mut set = HashSet::new();
        set.insert(String::from("m2")); // m2
        set.insert(String::from("m1")); // m1
        set.insert(String::from("m2"));
        set.insert(String::from("m2"));
        set.insert(String::from("m3")); // m3
        set.insert(String::from("m1"));
        set.insert(String::from("m4")); // m4
        set.insert(String::from("m5")); // m5
        set.insert(String::from("m1"));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock_1 = vec_strings!["key", "m1"];
        let buffer_mock_2 = vec_strings!["key", "m2"];

        let result_received_1 = Srem.run(buffer_mock_1, &mut database_mock);
        let result_received_2 = Srem.run(buffer_mock_2, &mut database_mock);

        let excepted_1 = RInteger::encode(1);
        let excepted_2 = RInteger::encode(1);
        assert_eq!(excepted_1, result_received_1.unwrap());
        assert_eq!(excepted_2, result_received_2.unwrap());
        if let TypeSaved::Set(set_post_srem) = database_mock.lock().unwrap().get("key").unwrap() {
            assert!(!set_post_srem.contains("m1")); // deleted
            assert!(!set_post_srem.contains("m2")); // deleted
            assert!(set_post_srem.contains("m3"));
            assert!(set_post_srem.contains("m4"));
            assert!(set_post_srem.contains("m5"));
            assert!(set_post_srem.len().eq(&3))
        };
    }
    #[test]
    fn test_02_srem_accepts_multiples_member_arguments_to_remove() {
        let mut set = HashSet::new();
        set.insert(String::from("m1")); // m1
        set.insert(String::from("m2")); // m2
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key", "m1901020", "m1", "m1", "m1", "m192192", "m1", "m1"];

        let result_received = Srem.run(buffer_mock, &mut database_mock);

        let excepted = RInteger::encode(1);
        assert_eq!(excepted, result_received.unwrap());
        if let TypeSaved::Set(set_post_srem) = database_mock.lock().unwrap().get("key").unwrap() {
            assert!(!set_post_srem.contains("m1")); // deleted one time
            assert!(set_post_srem.contains("m2"));
            assert!(set_post_srem.len().eq(&1))
        };
    }

    #[test]
    fn test_03_srem_return_zero_if_there_are_no_members_at_the_set_for_remove() {
        let mut set = HashSet::new();
        set.insert(String::from("m1")); // m1
        set.insert(String::from("m2")); // m2
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key", "m3", "m4"];

        let result_received = Srem.run(buffer_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
        if let TypeSaved::Set(set_post_srem) = database_mock.lock().unwrap().get("key").unwrap() {
            assert!(set_post_srem.contains("m1")); // unmodified
            assert!(set_post_srem.contains("m2")); // unmodified
            assert!(set_post_srem.len().eq(&2))
        };
    }

    #[test]
    fn test_04_srem_return_zero_if_key_does_not_exist_in_database() {
        let set = HashSet::new();
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key_random", "m1"];

        let result_received = Srem.run(buffer_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test_05_srem_return_error_wrongtype_if_execute_with_key_of_string() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock.lock().unwrap().insert(
            "keyOfString".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_mock = vec_strings!["keyOfString", "value"];

        let result_received = Srem.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test_06_srem_return_error_wrongtype_if_execute_with_key_of_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        database_mock
            .lock()
            .unwrap()
            .insert("keyOfList".to_string(), TypeSaved::List(new_list));

        let buffer_mock = vec_strings!["keyOfList", "value1", "value2"];

        let result_received = Srem.run(buffer_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test_07_srem_return_zero_if_set_is_empty() {
        let set = HashSet::new();
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::Set(set));
        let buffer_mock = vec_strings!["key", "value1", "value2"];

        let result_received = Srem.run(buffer_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
        if let TypeSaved::Set(set_post_srem) = database_mock.lock().unwrap().get("key").unwrap() {
            assert!(set_post_srem.len().eq(&0))
        };
    }
}
