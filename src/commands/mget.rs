use super::database_mock::DatabaseMock;
use crate::{
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RedisType},
};
pub struct Mget;

impl Mget {
    #[allow(unused_mut)]
    pub fn run(
        mut buffer_vec: Vec<&str>,
        database: &mut DatabaseMock,
    ) -> Result<String, ErrorStruct> {
        if buffer_vec.is_empty() {
            let message_error = redis_messages::not_empty_values_for("mget command");
            return Err(ErrorStruct::new(
                message_error.get_prefix(),
                message_error.get_message(),
            ));
        }

        if buffer_vec.len() == 1 {
            let error_message = redis_messages::wrong_number_args_for("mget");
            return Err(ErrorStruct::new(
                error_message.get_prefix(),
                error_message.get_message(),
            ));
        }

        let command = buffer_vec[0];
        if !command.eq("mget") {
            let concat_vector_buffer = buffer_vec.join(" ");
            let error_message = redis_messages::command_not_found_in(concat_vector_buffer);
            return Err(ErrorStruct::new(
                error_message.get_prefix(),
                error_message.get_message(),
            ));
        }

        buffer_vec.remove(0);
        let mut strings = database.get_mut_strings();
        let mut values_obtained: Vec<String> = Vec::new();
        buffer_vec.iter().for_each(|key| {
            if strings.contains_key(*key) {
                let value = strings.get(*key).unwrap();
                values_obtained.push(value.to_string());
            } else {
                values_obtained.push("(nil)".to_string());
            }
        });
        Ok(RArray::encode(values_obtained))
    }
}

#[cfg(test)]
mod test_get {
    use crate::{commands::set::Set, native_types::RArray};

    use super::*;

    #[test]
    fn test01_mget_value_of_key_correct_is_success() {
        let buffer_vec_mock_set1 = vec!["set", "key1", "value"];
        let buffer_vec_mock_set2 = vec!["set", "key2", "value"];
        let buffer_vec_mock_get = vec!["mget", "key2", "asd", "key1"];
        let mut database_mock = DatabaseMock::new();

        let _ = Set::run(buffer_vec_mock_set1, &mut database_mock);
        let _ = Set::run(buffer_vec_mock_set2, &mut database_mock);
        let result_received = Mget::run(buffer_vec_mock_get, &mut database_mock);

        // ->> "*3\r\n $5\r\nvalue\r\n $-1\r\n $5\r\nvalue\r\n"
        let excepted_vec = vec![
            "value".to_string(),
            "(nil)".to_string(),
            "value".to_string(),
        ];
        let excepted_vec_encoded = RArray::encode(excepted_vec);
        assert_eq!(excepted_vec_encoded, result_received.unwrap());
    }

    #[test]
    fn test02_mget_does_not_maintain_order() {
        let buffer_vec_mock_set1 = vec!["set", "key1", "value1"];
        let buffer_vec_mock_set2 = vec!["set", "key2", "value2"];
        let buffer_vec_mock_get1 = vec!["mget", "key2", "asd", "key1"];
        let buffer_vec_mock_get2 = vec!["mget", "asd", "key2", "key1"];
        let buffer_vec_mock_get3 = vec!["mget", "key1", "key2", "asd"];
        let mut database_mock = DatabaseMock::new();

        let _ = Set::run(buffer_vec_mock_set1, &mut database_mock);
        let _ = Set::run(buffer_vec_mock_set2, &mut database_mock);

        let result_received = Mget::run(buffer_vec_mock_get1, &mut database_mock);
        let excepted_vec = vec![
            "value2".to_string(),
            "(nil)".to_string(),
            "value1".to_string(),
        ];
        let excepted_vec_encoded = RArray::encode(excepted_vec);
        assert_eq!(excepted_vec_encoded, result_received.unwrap());

        let result_received = Mget::run(buffer_vec_mock_get2, &mut database_mock);
        let excepted_vec = vec![
            "(nil)".to_string(),
            "value2".to_string(),
            "value1".to_string(),
        ];
        let excepted_vec_encoded = RArray::encode(excepted_vec);
        assert_eq!(excepted_vec_encoded, result_received.unwrap());

        let result_received = Mget::run(buffer_vec_mock_get3, &mut database_mock);
        let excepted_vec = vec![
            "value1".to_string(),
            "value2".to_string(),
            "(nil)".to_string(),
        ];
        let excepted_vec_encoded = RArray::encode(excepted_vec);
        assert_eq!(excepted_vec_encoded, result_received.unwrap());
    }
}
