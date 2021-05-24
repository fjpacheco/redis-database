use std::collections::HashMap;
use crate::{database::TypesSaved, messages::redis_messages, native_types::{ErrorStruct, RSimpleString, RedisType}};
use crate::messages::redis_messages::command_not_found_in;
use crate::messages::redis_messages::not_empty_values_for;


pub mod redis_string {
  
    use crate::{messages::redis_messages::wrongtype_in_get_key, native_types::RBulkString};

    use super::*;

    pub fn run(buffer: String, database: &mut HashMap<String, TypesSaved>) -> Result<String, ErrorStruct> {
        if buffer.is_empty(){
            let message_error = not_empty_values_for("Redis strings");
            return Err(ErrorStruct::new(message_error.get_prefix(), message_error.get_message()))
        }

        let buffer_vec: Vec<&str>= buffer.split_whitespace().collect();
        let command = buffer_vec[0].to_lowercase();
       
        match command.as_str() {
            "set" => set(buffer_vec, database),
            "get" => get(buffer_vec, database),
            _ => {           
                let message_error = command_not_found_in(buffer);
                Err(ErrorStruct::new(message_error.get_prefix(), message_error.get_message()))
            }
        }
        
    }
    
    pub fn set(buffer_vec: Vec<&str>, database: &mut HashMap<String, TypesSaved>) -> Result<String, ErrorStruct> {
        if buffer_vec.is_empty(){
            let message_error = not_empty_values_for("Redis strings");
            return Err(ErrorStruct::new(message_error.get_prefix(), message_error.get_message()))
        }
        
        if buffer_vec.len() == 1 || buffer_vec.len() == 2 {
            let error_message = redis_messages::arguments_invalid_to(&buffer_vec[0].to_string());
            return Err(ErrorStruct::new(error_message.get_prefix(), error_message.get_message()))
        }

        let value = buffer_vec[2].to_string();
        let key = buffer_vec[1].to_string();

        let _ = database.insert(key, TypesSaved::String(value));    //  "Set" command does not interest Some/None of insert.
        
        Ok(RSimpleString::encode(redis_messages::ok()))
    }

    pub fn get(buffer_vec: Vec<&str>, database: &mut HashMap<String, TypesSaved>) -> Result<String, ErrorStruct> {
        if buffer_vec.is_empty(){
            let message_error = not_empty_values_for("Redis strings");
            return Err(ErrorStruct::new(message_error.get_prefix(), message_error.get_message()))
        }
        
        if buffer_vec.len() != 2 { 
            let error_message = redis_messages::wrong_number_args_for("get");
            return Err(ErrorStruct::new(error_message.get_prefix(), error_message.get_message()))
        }
        
        let key = buffer_vec[1].to_string();

        match database.get(&key) {
            Some(item) => {
                match item{
                    TypesSaved::String(item) => Ok(RBulkString::encode(item.to_string())),
                    _ => {
                            let message_error = wrongtype_in_get_key();
                            return Err(ErrorStruct::new(message_error.get_prefix(), message_error.get_message()))}
                }
            }
            None => Ok(RBulkString::encode(redis_messages::nil())),
        }
    }
}


#[cfg(test)]
mod test_decode {
    use crate::native_types::RBulkString;

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
        let excepted_result = ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, error_received_encoded);
    }

    #[test]
    fn test03_set_key_and_value_return_ok() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let result_received = redis_string::set(buffer_vec_mock, &mut database_mock);

        let excepted_result: String = ("+".to_owned() + &redis_messages::ok() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.unwrap());
    }

    #[test]
    fn test04_set_key_and_value_save_correctly_in_database_mock() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let _ = redis_string::set(buffer_vec_mock, &mut database_mock);
        let received = database_mock.get(&"key".to_string());

        let excepted = TypesSaved::String("value".to_string());
        assert_eq!(received.unwrap(), &excepted);
    }

    #[test]
    fn test05_set_key_and_value_but_get_another_key_return_none() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let _ = redis_string::set(buffer_vec_mock, &mut database_mock);
        let received = database_mock.get(&"key2".to_string());

        assert_eq!(received, None);
    }

    #[test]
    fn test06_set_without_value_return_err() {
        let buffer_vec_mock = vec!["set", "key"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let result_received = redis_string::set(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = redis_messages::arguments_invalid_to("set");
        let excepted_result: String = ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received_encoded);
    }

    #[test]
    fn test07_set_without_value_and_key_return_err() {
        let buffer_vec_mock = vec!["set"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let result_received = redis_string::set(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = redis_messages::arguments_invalid_to("set");
        let excepted_result = ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received_encoded);
    }
       
    #[test]
    fn test08_get_value_of_key_correct_is_success() {
        let buffer_vec_mock_set = vec!["set", "key", "value"];
        let buffer_vec_mock_get = vec!["get", "key"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let _ = redis_string::set(buffer_vec_mock_set, &mut database_mock);
        let result_received = redis_string::get(buffer_vec_mock_get, &mut database_mock);

        let excepted_result = RBulkString::encode("value".to_string());
        assert_eq!(excepted_result, result_received.unwrap());
    }

    #[test]
    fn test09_get_value_of_key_inorrect_return_result_ok_with_nil() {
        let buffer_vec_mock_set = vec!["set", "key", "value"];
        let buffer_vec_mock_get = vec!["get", "key_other"];
        let mut database_mock: HashMap<String, TypesSaved> = HashMap::new();

        let _ = redis_string::set(buffer_vec_mock_set, &mut database_mock);
        let result_received = redis_string::get(buffer_vec_mock_get, &mut database_mock);
        let received = result_received.unwrap();

        let excepted_result = "$-1\r\n".to_string();
        assert_eq!(excepted_result, received)        
    }

}

