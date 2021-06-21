use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RBulkString, RedisType},
};

use super::no_more_values;

pub struct Get;

impl Runnable<Database> for Get {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_empty(&&mut buffer_vec, "get")?;

        let key = buffer_vec.pop().unwrap().to_string();

        no_more_values(&buffer_vec, "get")?;

        match database.get(&key) {
            Some(item) => match item {
                TypeSaved::String(item) => Ok(RBulkString::encode(item.to_string())),
                _ => {
                    err_wrongtype!()
                }
            },
            None => Ok(RBulkString::encode(redis_messages::nil())),
        }
    }
}

#[cfg(test)]
mod test_get {

    use super::*;

    #[test]
    fn test01_get_value_of_key_correct_is_success() {
        let buffer_vec_mock_get = vec!["key"];
        let mut database_mock = Database::new();

        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let result_received = Get.run(buffer_vec_mock_get, &mut database_mock);

        let expected_result = RBulkString::encode("value".to_string());
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test02_get_value_of_key_inorrect_return_result_ok_with_nil() {
        let buffer_vec_mock_get = vec!["key_other"];
        let mut database_mock = Database::new();

        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let result_received = Get.run(buffer_vec_mock_get, &mut database_mock);
        let received = result_received.unwrap();

        let expected_result = "$-1\r\n".to_string();
        assert_eq!(expected_result, received)
    }
}
