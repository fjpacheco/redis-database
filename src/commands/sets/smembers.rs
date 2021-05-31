use crate::{
    commands::{
        check_empty_and_name_command,
        database_mock::{Database, TypeSaved},
    },
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RedisType},
};

pub struct Smembers;

impl Smembers {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let key = buffer_vec[1];

        match database.get_mut(key) {
            Some(item) => match item {
                TypeSaved::Set(a_set) => {
                    let vector: Vec<String> =
                        a_set.iter().map(|member| member.to_string()).collect();
                    Ok(RArray::encode(vector))
                }
                _ => {
                    let message_error = redis_messages::wrongtype();
                    Err(ErrorStruct::new(
                        message_error.get_prefix(),
                        message_error.get_message(),
                    ))
                }
            },
            None => Ok(RArray::encode(vec![])), // Empty array! => "*0\r\n"
        }
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty_and_name_command(&buffer_vec, "smembers")?;

    if buffer_vec.len() != 2 {
        let error_message = redis_messages::wrong_number_args_for("smembers");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_smembers_function {
    use std::collections::HashSet;

    use crate::{
        commands::{
            database_mock::{Database, TypeSaved},
            sets::smembers::Smembers,
        },
        messages::redis_messages,
        native_types::{RArray, RedisType},
    };

    #[test]
    fn test01_smembers_return_array_members_of_set_not_necessarily_ordered() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["smembers", "key"];

        let result_received = Smembers::run(buffer_vec_mock, &mut database_mock);
        let array = result_received.unwrap();

        // The Redis Sets are not necessarily ordered. That is why it is analyzed in lower level at Array Native Type.
        assert!(array.contains("*2\r\n"));
        assert!(array.contains("$2\r\nm1\r\n"));
        assert!(array.contains("$2\r\nm2\r\n"));
    }

    #[test]
    fn test02_smembers_return_an_empty_array_if_key_does_not_exist_in_database() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["smembers", "key_other"];

        let result_received = Smembers::run(buffer_vec_mock, &mut database_mock);

        let excepted = RArray::encode(vec![]); // Empty array! => "*0\r\n"
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test03_smembers_return_an_empty_array_if_set_is_empty() {
        let set = HashSet::new();
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["smembers", "key"];

        let result_received = Smembers::run(buffer_vec_mock, &mut database_mock);

        let excepted = RArray::encode(vec![]); // Empty array! => "*0\r\n"
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test04_smembers_return_error_wrongtype_if_execute_with_key_of_string() {
        let mut database_mock = Database::new();
        database_mock.insert(
            "keyOfString".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_vec_mock = vec!["smembers", "keyOfString"];

        let result_received = Smembers::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test05_smembers_return_error_wrongtype_if_execute_with_key_of_list() {
        let mut database_mock = Database::new();
        database_mock.insert(
            "keyOfList".to_string(),
            TypeSaved::List(vec!["value1".to_string(), "value2".to_string()]),
        );
        let buffer_vec_mock = vec!["smembers", "keyOfList"];

        let result_received = Smembers::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test06_smembers_return_error_arguments_invalid_if_buffer_has_many_one_arguments() {
        let mut database_mock = Database::new();
        let buffer_vec_mock = vec!["smembers", "arg1", "arg2", "arg3"];

        let result_received = Smembers::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("smembers");
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test07_smembers_return_error_arguments_invalid_if_buffer_dont_have_key() {
        let mut database_mock = Database::new();
        let buffer_vec_mock = vec!["smembers"];

        let result_received = Smembers::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("smembers");
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
