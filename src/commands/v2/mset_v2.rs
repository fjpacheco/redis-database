use crate::{
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

use super::database_mock_v2::{DatabaseMock2, TypeSaved};

pub struct Mset2;

impl Mset2 {
    #[allow(unused_mut)]
    pub fn run(
        mut buffer_vec: Vec<&str>,
        database: &mut DatabaseMock2,
    ) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let mut keys_and_value = buffer_vec.chunks(2);
        keys_and_value.into_iter().for_each(|x| {
            database.insert(x[0].to_string(), TypeSaved::String(x[1].to_string()));
        });

        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    if buffer_vec.is_empty() {
        let message_error = redis_messages::not_empty_values_for("mset command");
        return Err(ErrorStruct::new(
            message_error.get_prefix(),
            message_error.get_message(),
        ));
    }
    let command = buffer_vec.remove(0);
    if buffer_vec.is_empty() || is_odd(&*buffer_vec) {
        let error_message = redis_messages::wrong_number_args_for("mset");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }
    if !command.eq("mset") {
        let concat_vector_buffer = buffer_vec.join(" ");
        let error_message = redis_messages::command_not_found_in(concat_vector_buffer);
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

fn is_odd(buffer_vec: &[&str]) -> bool {
    buffer_vec.len() % 2 == 1
}

#[cfg(test)]
mod test_mset_function {
    use crate::{
        commands::v2::{
            database_mock_v2::DatabaseMock2, get_v2::Get2, mset_v2::Mset2, set_v2::Set2,
        },
        messages::redis_messages,
        native_types::{RBulkString, RedisType},
    };

    #[test]
    fn test01_mset_reemplace_value_old_of_key_and_insert_more_elements() {
        let buffer_vec_mock1 = vec!["set", "key1", "value1"];
        let buffer_vec_mock2 = vec!["mset", "key1", "value1_new", "key2", "value2"];
        let buffer_vec_mock_get1 = vec!["get", "key1"];
        let buffer_vec_mock_get2 = vec!["get", "key2"];
        let mut database_mock = DatabaseMock2::new();
        let _ = Set2::run(buffer_vec_mock1, &mut database_mock);

        let _ = Mset2::run(buffer_vec_mock2, &mut database_mock);

        let result_received1 = Get2::run(buffer_vec_mock_get1, &mut database_mock);
        let excepted = RBulkString::encode("value1_new".to_string());
        assert_eq!(excepted, result_received1.unwrap());

        let result_received2 = Get2::run(buffer_vec_mock_get2, &mut database_mock);
        let excepted = RBulkString::encode("value2".to_string());
        assert_eq!(excepted, result_received2.unwrap());
    }

    #[test]
    fn test02_mset_with_bad_args_return_err() {
        let buffer_vec_mock = vec!["mset", "key1", "value1_new", "key2"];
        let mut database_mock = DatabaseMock2::new();

        let result_received = Mset2::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_message_redis = redis_messages::wrong_number_args_for("mset");
        let excepted_result: String =
            ("-".to_owned() + &excepted_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(excepted_result, result_received_encoded);
    }
}
