use crate::{
    commands::{check_empty, Runnable},
    database::Database,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Copy;

impl Runnable for Copy {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let key_source = buffer_vec[0];
        let key_destinty = buffer_vec[1];

        if !database.contains_key(key_source) | database.contains_key(key_destinty) {
            Ok(RInteger::encode(0))
        } else {
            let value = database.get(key_source).unwrap().clone(); // Unwrap Reason: Value associated for Key Source exists!
            database.insert(key_destinty.to_string(), value);
            Ok(RInteger::encode(1))
        }
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty(&buffer_vec, "copy")?;

    if buffer_vec.len() != 2 {
        // never "copy" or "copy arg1"
        let error_message = redis_messages::arguments_invalid_to("copy");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_copy_function {

    use std::collections::{HashSet, LinkedList};

    use crate::database::TypeSaved;

    use super::*;

    #[test]
    fn test01_copy_value_string_of_key_source_existent_into_key_destiny_non_existent_return_success_one(
    ) {
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock_get = vec!["key", "key_new"];

        let result_received = Copy.run(buffer_vec_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(1);
        assert_eq!(expected_result, result_received.unwrap());

        if let TypeSaved::String(set_post_copy) = database_mock.get("key").unwrap() {
            assert!(set_post_copy.contains("value"));
        }

        if let TypeSaved::String(set_post_copy) = database_mock.get("key_new").unwrap() {
            assert!(set_post_copy.contains("value"));
        }
    }

    #[test]
    fn test02_copy_value_string_of_key_source_existent_into_key_destiny_existent_return_error_zero()
    {
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        database_mock.insert(
            "key_new".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_vec_mock_get = vec!["key", "key_new"];

        let result_received = Copy.run(buffer_vec_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(0);
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test03_copy_value_string_of_key_source_non_existent_into_key_destiny_non_existent_return_error_zero(
    ) {
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock_get = vec!["key_random", "key_new"];

        let result_received = Copy.run(buffer_vec_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(0);
        assert_eq!(expected_result, result_received.unwrap());
    }

    #[test]
    fn test04_copy_value_set_of_key_source_existent_into_key_destiny_non_existent_return_success_one(
    ) {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));

        let buffer_vec_mock_get = vec!["key", "key_new"];

        let result_received = Copy.run(buffer_vec_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(1);
        assert_eq!(expected_result, result_received.unwrap());

        if let TypeSaved::Set(set_post_copy) = database_mock.get("key").unwrap() {
            assert!(set_post_copy.contains("m1"));
            assert!(set_post_copy.contains("m2"));
            assert!(set_post_copy.len().eq(&2))
        }

        if let TypeSaved::Set(set_post_copy) = database_mock.get("key_new").unwrap() {
            assert!(set_post_copy.contains("m1"));
            assert!(set_post_copy.contains("m2"));
            assert!(set_post_copy.len().eq(&2))
        }
    }

    #[test]
    fn test06_copy_value_list_of_key_source_existent_into_key_destiny_non_existent_return_success_one(
    ) {
        let mut database_mock = Database::new();
        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        database_mock.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer_vec_mock_get = vec!["key", "key_new"];

        let result_received = Copy.run(buffer_vec_mock_get, &mut database_mock);

        let expected_result = RInteger::encode(1);
        assert_eq!(expected_result, result_received.unwrap());

        if let TypeSaved::List(set_post_copy) = database_mock.get("key").unwrap() {
            assert!(set_post_copy.contains(&"value1".to_string()));
            assert!(set_post_copy.contains(&"value2".to_string()));
            assert!(set_post_copy.len().eq(&2))
        }

        if let TypeSaved::List(set_post_copy) = database_mock.get("key_new").unwrap() {
            assert!(set_post_copy.contains(&"value1".to_string()));
            assert!(set_post_copy.contains(&"value2".to_string()));
            assert!(set_post_copy.len().eq(&2))
        }
    }
}
