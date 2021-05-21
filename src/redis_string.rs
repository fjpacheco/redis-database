use std::collections::HashMap;

use crate::{database::RedisTypes, native_types::NativeTypes};

#[derive(Debug)]
pub struct RedisString {
    value: String,
}

// @fjpacheco -- I'm not sure:
mod messages_redis_strings {
    pub const NOT_EMPTY: &'static str = "ERR RedisString not accept empty values"; 
    pub const OK: &'static str = "OK";
    pub const NOT_FOUND: &'static str = "ERR Command not found";
    pub const SET_ARGUGMENTS_INVALID: &'static str = "ERR wrong number of arguments for \'set\' command";
}

impl RedisString {
    #[allow(dead_code)]
    fn new(value: String) -> Result<Self, NativeTypes>{
        if value.is_empty() {
            Err(NativeTypes::new_error(messages_redis_strings::NOT_EMPTY))
        }else{
            Ok(RedisString {value})
        }
    }
    
    #[allow(dead_code)]
    fn as_native_type(&self) -> NativeTypes {
        NativeTypes::new_bulk_string(self.value.as_str())
    }

    pub fn run(buffer: String, database: &mut HashMap<String, RedisTypes>) -> Result<NativeTypes, NativeTypes> {
        let buffer_vec: Vec<&str>= buffer.split_whitespace().collect();
        let command = buffer_vec[0];
        match command {
            "set" => Self::set(buffer_vec, database),
            _ => Err(NativeTypes::new_error(messages_redis_strings::NOT_FOUND)),
        }
        
    }

    fn set(buffer_vec: Vec<&str>, database: &mut HashMap<String, RedisTypes>) -> Result<NativeTypes, NativeTypes> {
        if buffer_vec.len() == 1 || buffer_vec.len() == 2 {
            return Err(NativeTypes::new_error(messages_redis_strings::SET_ARGUGMENTS_INVALID))
        }

        let value = buffer_vec[2].to_string();
        let redis_string_to_save = Self::new(value)?; // The high power of '?' command !
        let key = buffer_vec[1].to_string();

        let _ = database.insert(key, RedisTypes::String(redis_string_to_save));    //  "Set" command does not interest Some/None of insert.
        
        Ok(NativeTypes::new_simple_string(messages_redis_strings::OK))
    }
}


#[cfg(test)]
mod test_decode {
    use super::*;

    #[test]
    fn test01_set_return_simple_string_with_ok() {
        let buffer_vec_mock = vec!["set", "key", "value"];
        let mut database_mock: HashMap<String, RedisTypes> = HashMap::new();

        let result_received = RedisString::set(buffer_vec_mock, &mut database_mock);

        let excepted_result = ("+".to_owned() + messages_redis_strings::OK + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.unwrap().encode().unwrap());
    }


    #[test]
    fn test02_set_without_value_return_err() {
        let buffer_vec_mock = vec!["set", "key"];
        let mut database_mock: HashMap<String, RedisTypes> = HashMap::new();

        let result_received = RedisString::set(buffer_vec_mock, &mut database_mock);

        let excepted_result = ("-".to_owned() + messages_redis_strings::SET_ARGUGMENTS_INVALID + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.unwrap_err().encode().unwrap());
    }

    #[test]
    fn test03_set_without_value_and_key_return_err() {
        let buffer_vec_mock = vec!["set"];
        let mut database_mock: HashMap<String, RedisTypes> = HashMap::new();

        let result_received = RedisString::set(buffer_vec_mock, &mut database_mock);

        let excepted_result = ("-".to_owned() + messages_redis_strings::SET_ARGUGMENTS_INVALID + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.unwrap_err().encode().unwrap());
    }

    #[test]
    fn test04_redis_string_not_accept_empty_values() {
        let result_received = RedisString::new("".to_string());
   
        let excepted_result = ("-".to_owned() + messages_redis_strings::NOT_EMPTY + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.unwrap_err().encode().unwrap());
    }
    #[test]
    fn test05_redis_string_can_return_the_value_stored_as_bulk_string_native_type() {
        let value = "a value";
        let redis_string = RedisString::new(value.to_string()).unwrap();
   
        let result_received = redis_string.as_native_type();

        let excepted_result = ("$".to_owned() + value.len().to_string().as_str() + "\r\n" + value + "\r\n").to_string();
        assert_eq!(excepted_result, result_received.encode().unwrap());
    }
}

