use crate::{
    commands::{
        check_empty_and_name_command,
        database_mock::{DatabaseMock, TypeSaved},
        Runnable,
    },
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Scard;

impl Runnable for Scard {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut DatabaseMock,
    ) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;
        let key = buffer_vec[1];

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
    check_empty_and_name_command(&buffer_vec, "scard")?;

    if buffer_vec.len() != 2 {
        let error_message = redis_messages::wrong_number_args_for("scard");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod test_scard_function {
    use std::collections::{HashSet, LinkedList};

    use super::*;

    #[test]
    fn test01_scard_return_number_of_set_members() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        set.insert(String::from("m3"));
        let mut database_mock = DatabaseMock::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["scard", "key"];

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
        let mut database_mock = DatabaseMock::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["scard", "key"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(3);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test03_scard_return_zero_if_the_set_is_empty() {
        let set: HashSet<String> = HashSet::new();
        let mut database_mock = DatabaseMock::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["scard", "key"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test04_scard_return_zero_if_the_set_dont_exist() {
        let set: HashSet<String> = HashSet::new();
        let mut database_mock = DatabaseMock::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["scard", "key_random"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test05_scard_return_error_wrongtype_if_execute_with_key_of_string() {
        let mut database_mock = DatabaseMock::new();
        database_mock.insert(
            "keyOfString".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_vec_mock = vec!["scard", "keyOfString"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test06_scard_return_error_wrongtype_if_execute_with_key_of_lists() {
        let mut database_mock = DatabaseMock::new();
        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        database_mock.insert("keyOfList".to_string(), TypeSaved::List(new_list));
        let buffer_vec_mock = vec!["scard", "keyOfList"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test07_scard_return_error_arguments_invalid_if_buffer_has_many_one_key() {
        let mut database_mock = DatabaseMock::new();
        let buffer_vec_mock = vec!["scard", "key1", "key2", "key3"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("scard");
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test07_scard_return_error_arguments_invalid_if_buffer_dont_have_key() {
        let mut database_mock = DatabaseMock::new();
        let buffer_vec_mock = vec!["scard"];

        let result_received = Scard.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("scard");
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
