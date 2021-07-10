use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RedisType},
};

pub struct Mget;

impl Runnable<Database> for Mget {
    fn run(&self, buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_error_cases(&buffer)?;

        let mut values_obtained: Vec<String> = Vec::new();
        buffer
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

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    check_empty(buffer, "mget")?;

    if buffer.len() == 1 {
        // never "mget" alone
        let error_message = redis_messages::arguments_invalid_to("mget");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_get {

    use crate::vec_strings;

    use super::*;

    #[test]
    fn test01_mget_value_of_key_correct_is_success() {
        let buffer_mock_get = vec_strings!["key2", "asd", "key1"];
        let mut database_mock = Database::new();

        database_mock.insert("key1".to_string(), TypeSaved::String("value1".to_string()));
        database_mock.insert("key2".to_string(), TypeSaved::String("value2".to_string()));
        let result_received = Mget.run(buffer_mock_get, &mut database_mock);

        // ->> "*3\r\n $5\r\nvalue\r\n $-1\r\n $5\r\nvalue\r\n"
        let expected_vec = vec![
            "value2".to_string(),
            "(nil)".to_string(),
            "value1".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());
    }

    #[test]
    fn test02_mget_does_not_maintain_order() {
        let buffer_mock_get1 = vec_strings!["key2", "asd", "key1"];
        let buffer_mock_get2 = vec_strings!["asd", "key2", "key1"];
        let buffer_mock_get3 = vec_strings!["key1", "key2", "asd"];
        let mut database_mock = Database::new();

        database_mock.insert("key1".to_string(), TypeSaved::String("value1".to_string()));
        database_mock.insert("key2".to_string(), TypeSaved::String("value2".to_string()));

        let result_received = Mget.run(buffer_mock_get1, &mut database_mock);
        let expected_vec = vec![
            "value2".to_string(),
            "(nil)".to_string(),
            "value1".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());

        let result_received = Mget.run(buffer_mock_get2, &mut database_mock);
        let expected_vec = vec![
            "(nil)".to_string(),
            "value2".to_string(),
            "value1".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());

        let result_received = Mget.run(buffer_mock_get3, &mut database_mock);
        let expected_vec = vec![
            "value1".to_string(),
            "value2".to_string(),
            "(nil)".to_string(),
        ];
        let expected_vec_encoded = RArray::encode(expected_vec);
        assert_eq!(expected_vec_encoded, result_received.unwrap());
    }
}
