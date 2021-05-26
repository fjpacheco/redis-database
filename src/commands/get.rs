use super::database_mock::DatabaseMock;
use crate::{
    messages::redis_messages,
    native_types::{ErrorStruct, RBulkString, RedisType},
};
pub struct Get;

impl Get {
    #[allow(unused_mut)]
    pub fn run(
        mut buffer_vec: Vec<&str>,
        database: &mut DatabaseMock,
    ) -> Result<String, ErrorStruct> {
        if buffer_vec.is_empty() {
            let message_error = redis_messages::not_empty_values_for("Redis strings");
            return Err(ErrorStruct::new(
                message_error.get_prefix(),
                message_error.get_message(),
            ));
        }

        if buffer_vec.len() != 2 {
            let error_message = redis_messages::wrong_number_args_for("get");
            return Err(ErrorStruct::new(
                error_message.get_prefix(),
                error_message.get_message(),
            ));
        }

        let key = buffer_vec[1].to_string();
        let mut strings = database.get_mut_strings();

        match strings.get(&key) {
            Some(item) => Ok(RBulkString::encode(item.to_string())),
            None => {
                if database.contains_key(&key) {
                    let message_error = redis_messages::wrongtype_in_get_key();
                    Err(ErrorStruct::new(
                        message_error.get_prefix(),
                        message_error.get_message(),
                    ))
                } else {
                    Ok(RBulkString::encode(redis_messages::nil()))
                }
            }
        }
    }
}

#[cfg(test)]
mod test_get {
    use crate::commands::set::Set;

    use super::*;

    #[test]
    fn test01_get_value_of_key_correct_is_success() {
        let buffer_vec_mock_set = vec!["set", "key", "value"];
        let buffer_vec_mock_get = vec!["get", "key"];
        let mut database_mock = DatabaseMock::new();

        let _ = Set::run(buffer_vec_mock_set, &mut database_mock);
        let result_received = Get::run(buffer_vec_mock_get, &mut database_mock);

        let excepted_result = RBulkString::encode("value".to_string());
        assert_eq!(excepted_result, result_received.unwrap());
    }

    #[test]
    fn test02_get_value_of_key_inorrect_return_result_ok_with_nil() {
        let buffer_vec_mock_set = vec!["set", "key", "value"];
        let buffer_vec_mock_get = vec!["get", "key_other"];
        let mut database_mock = DatabaseMock::new();

        let _ = Set::run(buffer_vec_mock_set, &mut database_mock);
        let result_received = Get::run(buffer_vec_mock_get, &mut database_mock);
        let received = result_received.unwrap();

        let excepted_result = "$-1\r\n".to_string();
        assert_eq!(excepted_result, received)
    }
}
