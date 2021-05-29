use crate::{
    commands::check_empty_and_name_command,
    database::{Database, TypeSaved},
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RedisType},
};

pub struct Mget;

impl Mget {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        buffer_vec.remove(0);
        let mut values_obtained: Vec<String> = Vec::new();
        buffer_vec
            .iter()
            .for_each(|key| match database.get(&key.to_string()) {
                Some(value) => match value {
                    TypeSaved::String(value) => values_obtained.push(value.to_string()),
                    _ => values_obtained.push("(nil)".to_string()),
                },
                None => {
                    values_obtained.push("(nil)".to_string());
                }
            });
        Ok(RArray::encode(values_obtained))
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty_and_name_command(&buffer_vec, "mget")?;

    if buffer_vec.len() == 1 {
        // never "mget" alone
        let error_message = redis_messages::wrong_number_args_for("mget");
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
    fn test01_mget_value_of_key_correct_is_success() {
        let buffer_vec_mock_set1 = vec!["set", "key1", "value"];
        let buffer_vec_mock_set2 = vec!["set", "key2", "value"];
        let buffer_vec_mock_get = vec!["mget", "key2", "asd", "key1"];
        let mut database_mock = Database::new();

        let _ = Set::run(buffer_vec_mock_set1, &mut database_mock);
        let _ = Set::run(buffer_vec_mock_set2, &mut database_mock);
        let result_received = Mget::run(buffer_vec_mock_get, &mut database_mock);

        // ->> "*3\r\n $5\r\nvalue\r\n $-1\r\n $5\r\nvalue\r\n"
        let expected_vec = vec![
            "value".to_string(),
            "(nil)".to_string(),
            "value".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());
    }

    #[test]
    fn test02_mget_does_not_maintain_order() {
        let buffer_vec_mock_set1 = vec!["set", "key1", "value1"];
        let buffer_vec_mock_set2 = vec!["set", "key2", "value2"];
        let buffer_vec_mock_get1 = vec!["mget", "key2", "asd", "key1"];
        let buffer_vec_mock_get2 = vec!["mget", "asd", "key2", "key1"];
        let buffer_vec_mock_get3 = vec!["mget", "key1", "key2", "asd"];
        let mut database_mock = Database::new();

        let _ = Set::run(buffer_vec_mock_set1, &mut database_mock);
        let _ = Set::run(buffer_vec_mock_set2, &mut database_mock);

        let result_received = Mget::run(buffer_vec_mock_get1, &mut database_mock);
        let expected_vec = vec![
            "value2".to_string(),
            "(nil)".to_string(),
            "value1".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());

        let result_received = Mget::run(buffer_vec_mock_get2, &mut database_mock);
        let expected_vec = vec![
            "(nil)".to_string(),
            "value2".to_string(),
            "value1".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());

        let result_received = Mget::run(buffer_vec_mock_get3, &mut database_mock);
        let expected_vec = vec![
            "value1".to_string(),
            "value2".to_string(),
            "(nil)".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());
    }
}
