use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Srem;

impl Runnable<Database> for Srem {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let key = buffer_vec[0];

        match database.get_mut(key) {
            Some(item) => match item {
                TypeSaved::Set(item) => {
                    let count_deleted = buffer_vec
                        .iter()
                        .skip(1)
                        .map(|member| item.remove(*member))
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

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty(&buffer_vec, "srem")?;

    if buffer_vec.len() < 2 {
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
    use std::collections::{HashSet, VecDeque};

    use super::*;

    #[test]
    fn test01_srem_remove_members_of_set_and_return_the_eliminated_amount() {
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
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock_1 = vec!["key", "m1"];
        let buffer_vec_mock_2 = vec!["key", "m2"];

        let result_received_1 = Srem.run(buffer_vec_mock_1, &mut database_mock);
        let result_received_2 = Srem.run(buffer_vec_mock_2, &mut database_mock);

        let excepted_1 = RInteger::encode(1);
        let excepted_2 = RInteger::encode(1);
        assert_eq!(excepted_1, result_received_1.unwrap());
        assert_eq!(excepted_2, result_received_2.unwrap());
        if let TypeSaved::Set(set_post_srem) = database_mock.get("key").unwrap() {
            assert!(!set_post_srem.contains("m1")); // deleted
            assert!(!set_post_srem.contains("m2")); // deleted
            assert!(set_post_srem.contains("m3"));
            assert!(set_post_srem.contains("m4"));
            assert!(set_post_srem.contains("m5"));
            assert!(set_post_srem.len().eq(&3))
        }
    }
    #[test]
    fn test02_srem_accepts_multiples_member_arguments_to_remove() {
        let mut set = HashSet::new();
        set.insert(String::from("m1")); // m1
        set.insert(String::from("m2")); // m2
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key", "m1901020", "m1", "m1", "m1", "m192192", "m1", "m1"];

        let result_received = Srem.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(1);
        assert_eq!(excepted, result_received.unwrap());
        if let TypeSaved::Set(set_post_srem) = database_mock.get("key").unwrap() {
            assert!(!set_post_srem.contains("m1")); // deleted one time
            assert!(set_post_srem.contains("m2"));
            assert!(set_post_srem.len().eq(&1))
        }
    }

    #[test]
    fn test03_srem_return_zero_if_there_are_no_members_at_the_set_for_remove() {
        let mut set = HashSet::new();
        set.insert(String::from("m1")); // m1
        set.insert(String::from("m2")); // m2
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key", "m3", "m4"];

        let result_received = Srem.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
        if let TypeSaved::Set(set_post_srem) = database_mock.get("key").unwrap() {
            assert!(set_post_srem.contains("m1")); // unmodified
            assert!(set_post_srem.contains("m2")); // unmodified
            assert!(set_post_srem.len().eq(&2))
        }
    }

    #[test]
    fn test04_srem_return_zero_if_key_does_not_exist_in_database() {
        let set = HashSet::new();
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key_random", "m1"];

        let result_received = Srem.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test05_srem_return_error_wrongtype_if_execute_with_key_of_string() {
        let mut database_mock = Database::new();
        database_mock.insert(
            "keyOfString".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_vec_mock = vec!["keyOfString", "value"];

        let result_received = Srem.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test06_srem_return_error_wrongtype_if_execute_with_key_of_list() {
        let mut database_mock = Database::new();
        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        database_mock.insert("keyOfList".to_string(), TypeSaved::List(new_list));

        let buffer_vec_mock = vec!["keyOfList", "value1", "value2"];

        let result_received = Srem.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test07_srem_return_zero_if_set_is_empty() {
        let set = HashSet::new();
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key", "value1", "value2"];

        let result_received = Srem.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
        if let TypeSaved::Set(set_post_srem) = database_mock.get("key").unwrap() {
            assert!(set_post_srem.len().eq(&0))
        }
    }
}
