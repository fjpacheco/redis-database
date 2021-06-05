use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Scard;

impl Runnable for Scard {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let key = buffer_vec[0];

        match database.get_mut(key) {
            Some(item) => match item {
                TypeSaved::Set(item) => Ok(RInteger::encode(item.len() as isize)),
                _ => {
                    err_wrongtype!()
                }
            },
            None => Ok(RInteger::encode(0)),
        }
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty(&buffer_vec, "scard")?;

    if buffer_vec.len() != 1 {
        let error_message = redis_messages::arguments_invalid_to("scard");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod test_scard_function {
    use std::collections::{HashSet, VecDeque};

    use super::*;

    #[test]
    fn test01_scard_return_number_of_set_members() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        set.insert(String::from("m3"));
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(3);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test02_scard_return_number_of_set_members_but_not_repeated() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        set.insert(String::from("m3"));
        set.insert(String::from("m3"));
        set.insert(String::from("m3"));
        set.insert(String::from("m2"));
        set.insert(String::from("m2"));
        set.insert(String::from("m1"));
        set.insert(String::from("m1"));
        set.insert(String::from("m3"));
        set.insert(String::from("m3"));
        set.insert(String::from("m3"));
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(3);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test03_scard_return_zero_if_the_set_is_empty() {
        let set: HashSet<String> = HashSet::new();
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test04_scard_return_zero_if_the_set_dont_exist() {
        let set: HashSet<String> = HashSet::new();
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key_random"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test05_scard_return_error_wrongtype_if_execute_with_key_of_string() {
        let mut database_mock = Database::new();
        database_mock.insert(
            "keyOfString".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_vec_mock = vec!["keyOfString"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test06_scard_return_error_wrongtype_if_execute_with_key_of_lists() {
        let mut database_mock = Database::new();
        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        database_mock.insert("keyOfList".to_string(), TypeSaved::List(new_list));
        let buffer_vec_mock = vec!["keyOfList"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test07_scard_return_error_arguments_invalid_if_buffer_has_many_one_key() {
        let mut database_mock = Database::new();
        let buffer_vec_mock = vec!["key1", "key2", "key3"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("scard");
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
