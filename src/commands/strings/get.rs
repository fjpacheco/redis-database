use crate::{
    commands::check_empty_and_name_command,
    database::{Database, TypeSaved},
    messages::redis_messages,
    native_types::{ErrorStruct, RBulkString, RedisType},
};

pub struct Get;

impl Get {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let key = buffer_vec[1].to_string();

        match database.get(&key) {
            Some(item) => match item {
                TypeSaved::String(item) => Ok(RBulkString::encode(item.to_string())),
                _ => {
                    let message_error = redis_messages::wrongtype_in_get_key();
                    Err(ErrorStruct::new(
                        message_error.get_prefix(),
                        message_error.get_message(),
                    ))
                }
            },
            None => Ok(RBulkString::encode(redis_messages::nil())),
        }
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty_and_name_command(&buffer_vec, "get")?;

    if buffer_vec.len() != 2 {
        // only "get key"
        let error_message = redis_messages::wrong_number_args_for("get");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_get {

    use crate::commands::strings::set::Set;

    use super::*;

    #[test]
    fn test01_get_value_of_key_correct_is_success() {
        let buffer_vec_mock_set = vec!["set", "key", "value"];
        let buffer_vec_mock_get = vec!["get", "key"];
        let mut database_mock = Database::new();

        let _ = Set::run(buffer_vec_mock_set, &mut database_mock);
        let result_received = Get::run(buffer_vec_mock_get, &mut database_mock);

        let expected_result = RBulkString::encode("value".to_string());
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test02_get_value_of_key_inorrect_return_result_ok_with_nil() {
        let buffer_vec_mock_set = vec!["set", "key", "value"];
        let buffer_vec_mock_get = vec!["get", "key_other"];
        let mut database_mock = Database::new();

        let _ = Set::run(buffer_vec_mock_set, &mut database_mock);
        let result_received = Get::run(buffer_vec_mock_get, &mut database_mock);
        let received = result_received.unwrap();

        let expected_result = "$-1\r\n".to_string();
        assert_eq!(expected_result, received)
    }
}
