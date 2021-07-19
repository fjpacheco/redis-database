use crate::{
    commands::Runnable,
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RBulkString, RedisType},
};

pub struct Get;

impl Runnable<Database> for Get {
    fn run(&self, buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_error_cases(&buffer)?;

        let key = buffer[0].to_string();

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

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    if buffer.len() != 1 {
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
    use crate::commands::create_notifier;

    use crate::vec_strings;

    use super::*;

    #[test]
    fn test01_get_value_of_key_correct_is_success() {
        let buffer_mock_get = vec_strings!["key"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Database::new(notifier);

        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let result_received = Get.run(buffer_mock_get, &mut database_mock);

        let expected_result = RBulkString::encode("value".to_string());
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test02_get_value_of_key_inorrect_return_result_ok_with_nil() {
        let buffer_mock_get = vec_strings!["key_other"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Database::new(notifier);

        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let result_received = Get.run(buffer_mock_get, &mut database_mock);
        let received = result_received.unwrap();

        let expected_result = "$-1\r\n".to_string();
        assert_eq!(expected_result, received)
    }
}
