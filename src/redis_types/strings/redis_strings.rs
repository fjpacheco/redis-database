use crate::database::TypesSaved;
use crate::messages::redis_messages::command_not_found_in;
use crate::messages::redis_messages::not_empty_values_for;
use crate::native_types::ErrorStruct;
use std::collections::HashMap;

pub mod redis_string {

    use crate::redis_types::strings::commands::{Get, Set};

    use super::*;

    pub fn run(
        buffer: String,
        database: &mut HashMap<String, TypesSaved>,
    ) -> Result<String, ErrorStruct> {
        if buffer.is_empty() {
            let message_error = not_empty_values_for("Redis strings");
            return Err(ErrorStruct::new(
                message_error.get_prefix(),
                message_error.get_message(),
            ));
        }

        let buffer_vec: Vec<&str> = buffer.split_whitespace().collect();
        let command = buffer_vec[0].to_lowercase();

        match command.as_str() {
            "set" => Set::run(buffer_vec, database),
            "get" => Get::run(buffer_vec, database),
            _ => {
                let message_error = command_not_found_in(buffer);
                Err(ErrorStruct::new(
                    message_error.get_prefix(),
                    message_error.get_message(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod test_redis_strings {

    use crate::messages::redis_messages;

    use super::*;

    #[test]
    fn test01_run_with_buffer_of_set_return_simple_string_with_ok() {
        let buffer_mock = String::from("set key value");
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let result_received = redis_string::run(buffer_mock, &mut database_mock);

        let excepted_result: String = ("+".to_owned() + &redis_messages::ok() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.unwrap());
    }

    #[test]
    fn test02_run_does_not_accept_empty_values() {
        let empty_string = String::from("");
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let result_received = redis_string::run(empty_string, &mut database_mock);
        let error_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = not_empty_values_for("Redis strings");
        let excepted_result =
            ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, error_received_encoded);
    }
}
