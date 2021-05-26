use crate::{
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

use super::database_mock_v2::{DatabaseMock2, TypeSaved};

pub struct Set2;

impl Set2 {
    #[allow(unused_mut)]
    pub fn run(
        mut buffer_vec: Vec<&str>,
        database: &mut DatabaseMock2,
    ) -> Result<String, ErrorStruct> {
        if buffer_vec.is_empty() {
            let message_error = redis_messages::not_empty_values_for("Strings");
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

        if buffer_vec.len() >= 4 {
            let error_message = redis_messages::syntax_error();
            return Err(ErrorStruct::new(
                error_message.get_prefix(),
                error_message.get_message(),
            ));
        }

        let value = buffer_vec[2].to_string();
        let key = buffer_vec[1].to_string();

        let fields = database.get_mut_elements();
        let _ = fields.insert(key, TypeSaved::String(value)); // Reemplazo cualquier valor que haya antiguamente con ésa key.
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}

#[cfg(test)]
mod test_set_function {

    use crate::{commands::get_v2::Get2, native_types::RBulkString};

    use super::*;
    #[test]
    fn test01_set_key_and_value_return_ok() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let mut database_mock = DatabaseMock2::new();

        let result_received = Set2::run(buffer_vec_mock, &mut database_mock);

        let excepted_result: String = ("+".to_owned() + &redis_messages::ok() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.unwrap());
    }

    #[test]
    fn test02_set_key_and_value_save_correctly_in_database_mock() {
        let buffer_vec_mock_set = vec!["set", "key", "value"];
        let buffer_vec_mock_get = vec!["get", "key"];
        let mut database_mock = DatabaseMock2::new();

        let _ = Set2::run(buffer_vec_mock_set, &mut database_mock);
        let result_received = Get2::run(buffer_vec_mock_get, &mut database_mock);

        let excepted = RBulkString::encode("value".to_string());
        assert_eq!(result_received.unwrap(), excepted);
    }

    #[test]
    fn test03_set_key_and_value_but_get_another_key_return_none() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let buffer_vec_mock_get = vec!["get", "key2"];
        let mut database_mock = DatabaseMock2::new();

        let _ = Set2::run(buffer_vec_mock, &mut database_mock);
        let received = Get2::run(buffer_vec_mock_get, &mut database_mock);

        let excepted = RBulkString::encode("(nil)".to_string());
        assert_eq!(received.unwrap(), excepted);
    }

    #[test]
    fn test04_set_without_value_return_err() {
        let buffer_vec_mock = vec!["set", "key"];
        let mut database_mock = DatabaseMock2::new();

        let result_received = Set2::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = redis_messages::arguments_invalid_to("set");
        let excepted_result: String =
            ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received_encoded);
    }

    #[test]
    fn test05_set_without_value_and_key_return_err() {
        let buffer_vec_mock = vec!["set"];
        let mut database_mock = DatabaseMock2::new();

        let result_received = Set2::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = redis_messages::arguments_invalid_to("set");
        let excepted_result =
            ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received_encoded);
    }

    #[test]
    fn test06_set_without_value_and_key_return_err_syntax() {
        let buffer_vec_mock = vec!["set", "set", "set", "set"];
        let mut database_mock = DatabaseMock2::new();

        let result_received = Set2::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = redis_messages::syntax_error();
        let excepted_result =
            ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received_encoded);
    }
}
