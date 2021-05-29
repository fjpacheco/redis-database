use crate::{
    commands::check_empty_and_name_command,
    database::{Database, TypeSaved},
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct Mset;

impl Mset {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let keys_and_value = buffer_vec.chunks(2);
        keys_and_value.into_iter().for_each(|pair_key_value| {
            database.insert(
                pair_key_value[0].to_string(),
                TypeSaved::String(pair_key_value[1].to_string()),
            );
        });

        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty_and_name_command(&buffer_vec, "mset")?;

    buffer_vec.remove(0);
    if buffer_vec.is_empty() || is_odd(&*buffer_vec) {
        // never odd => "key1 value1 key2 value2 ...""
        let error_message = redis_messages::wrong_number_args_for("mset");
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
        commands::strings::{get::Get, mset::Mset, set::Set},
        database::Database,
        messages::redis_messages,
        native_types::{RBulkString, RedisType},
    };

    #[test]
    fn test01_mset_reemplace_value_old_of_key_and_insert_more_elements() {
        let buffer_vec_mock1 = vec!["set", "key1", "value1"];
        let buffer_vec_mock2 = vec!["Mset", "key1", "value1_new", "key2", "value2"];
        let buffer_vec_mock_get1 = vec!["get", "key1"];
        let buffer_vec_mock_get2 = vec!["get", "key2"];
        let mut database_mock = Database::new();
        let _ = Set::run(buffer_vec_mock1, &mut database_mock);

        let _ = Mset::run(buffer_vec_mock2, &mut database_mock);

        let result_received1 = Get::run(buffer_vec_mock_get1, &mut database_mock);
        let expected = RBulkString::encode("value1_new".to_string());
        assert_eq!(expected, result_received1.unwrap());

        let result_received2 = Get::run(buffer_vec_mock_get2, &mut database_mock);
        let expected = RBulkString::encode("value2".to_string());
        assert_eq!(expected, result_received2.unwrap());
    }

    #[test]
    fn test02_mset_with_bad_args_return_err() {
        let buffer_vec_mock = vec!["Mset", "key1", "value1_new", "key2"];
        let mut database_mock = Database::new();

        let result_received = Mset::run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrong_number_args_for("mset");
        let expected_result: String =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
