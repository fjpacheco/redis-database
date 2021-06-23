use crate::{commands::{Runnable, check_empty}, database::{Database, TypeSaved}, messages::redis_messages, native_types::{ErrorStruct, RSimpleString, RedisType}};


pub struct Set;

impl Runnable<Database> for Set {
    fn run(
        &self,
        mut buffer: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer)?;

        let value = buffer[1].to_string();
        let key = buffer[0].to_string();

        database.insert(key, TypeSaved::String(value)); // replace any old value with this key
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty(&buffer_vec, "set")?;

    if buffer_vec.len() == 1 {
        // never "arg1"
        let error_message = redis_messages::arguments_invalid_to("set");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    // Different error output => checked with src/redis-server!!
    if buffer_vec.len() != 2 {
        // never "arg1 arg2 arg3 ... "
        let error_message = redis_messages::syntax_error(); 
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}


#[cfg(test)]
mod test_set_function {

    use crate::native_types::RBulkString;

    use super::*;
    #[test]
    fn test01_set_key_and_value_return_ok() {
        let buffer_vec_mock = vec!["key", "value"];
        let mut database_mock = Database::new();

        let result_received = Set.run(buffer_vec_mock, &mut database_mock);

        let expected_result: String = ("+".to_owned() + &redis_messages::ok() + "\r\n").to_string();
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test02_set_key_and_value_save_correctly_in_database_mock() {
        let buffer_vec_mock_set = vec!["key", "value"];
        let mut database_mock = Database::new();

        let _ = Set.run(buffer_vec_mock_set, &mut database_mock);
        let mut get_received = String::new();
        if let TypeSaved::String(item) = database_mock.get("key").unwrap() {
            get_received = RBulkString::encode(item.to_string());
        }

        let expected = RBulkString::encode("value".to_string());
        assert_eq!(expected, get_received);
    }

    #[test]
    fn test03_set_key_and_value_but_get_another_key_return_none() {
        let buffer_vec_mock = vec!["key", "value"];
        let mut database_mock = Database::new();

        let _ = Set.run(buffer_vec_mock, &mut database_mock);
        let mut get_received = String::new();
        if let TypeSaved::String(item) = database_mock
            .get("key2")
            .unwrap_or(&TypeSaved::String("(nil)".to_string()))
        {
            get_received = RBulkString::encode(item.to_string());
        }

        let expected = RBulkString::encode("(nil)".to_string());
        assert_eq!(expected, get_received);
    }

    #[test]
    fn test04_set_without_value_and_key_return_err_syntax() {
        let buffer_vec_mock = vec!["set", "set", "set", "set"];
        let mut database_mock = Database::new();

        let result_received = Set.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::syntax_error();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
