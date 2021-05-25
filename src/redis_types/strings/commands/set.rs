use std::collections::HashMap;

use crate::{
    database::TypesSaved,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct Set;

impl Set {
    pub fn run(
        buffer_vec: Vec<&str>,
        database: &mut HashMap<String, TypesSaved>,
    ) -> Result<String, ErrorStruct> {
        if buffer_vec.is_empty() {
            let message_error = redis_messages::not_empty_values_for("Redis strings");
            return Err(ErrorStruct::new(
                message_error.get_prefix(),
                message_error.get_message(),
            ));
        }

        if buffer_vec.len() == 1 || buffer_vec.len() == 2 {
            let error_message = redis_messages::arguments_invalid_to(&buffer_vec[0].to_string());
            return Err(ErrorStruct::new(
                error_message.get_prefix(),
                error_message.get_message(),
            ));
        }

        let value = buffer_vec[2].to_string();
        let key = buffer_vec[1].to_string();

        let _ = database.insert(key, TypesSaved::String(value)); //  "Set" command does not interest Some/None of insert.

        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}

#[cfg(test)]
mod test_set {
    use super::*;
    #[test]
    fn test01_set_key_and_value_return_ok() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let result_received = Set::run(buffer_vec_mock, &mut database_mock);

        let excepted_result: String = ("+".to_owned() + &redis_messages::ok() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.unwrap());
    }

    #[test]
    fn test02_set_key_and_value_save_correctly_in_database_mock() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let _ = Set::run(buffer_vec_mock, &mut database_mock);
        let received = database_mock.get(&"key".to_string());

        let excepted = TypesSaved::String("value".to_string());
        assert_eq!(received.unwrap(), &excepted);
    }

    #[test]
    fn test03_set_key_and_value_but_get_another_key_return_none() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let _ = Set::run(buffer_vec_mock, &mut database_mock);
        let received = database_mock.get(&"key2".to_string());

        assert_eq!(received, None);
    }

    #[test]
    fn test04_set_without_value_return_err() {
        let buffer_vec_mock = vec!["set", "key"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let result_received = Set::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = redis_messages::arguments_invalid_to("set");
        let excepted_result: String =
            ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received_encoded);
    }

    #[test]
    fn test05_set_without_value_and_key_return_err() {
        let buffer_vec_mock = vec!["set"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let result_received = Set::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = redis_messages::arguments_invalid_to("set");
        let excepted_result =
            ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received_encoded);
    }
}
