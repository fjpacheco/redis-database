use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Sismember;

impl Runnable<Database> for Sismember {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let key = buffer_vec[0];

        match database.get_mut(key) {
            Some(item) => match item {
                TypeSaved::Set(item) => {
                    let member = buffer_vec[1];
                    let result = match item.contains(member) {
                        true => 1,
                        false => 0,
                    };
                    Ok(RInteger::encode(result))
                }
                _ => {
                    err_wrongtype!()
                }
            },
            None => Ok(RInteger::encode(0)),
        }
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty(&buffer_vec, "sismember")?;

    if buffer_vec.len() != 2 {
        let error_message = redis_messages::arguments_invalid_to("sismember");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod test_sismember_function {
    use std::collections::{HashSet, VecDeque};

    use super::*;

    #[test]
    fn test01_sismember_return_number_one_if_member_is_contained_in_set() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        set.insert(String::from("m2"));
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock_1 = vec!["key", "m1"];
        let buffer_vec_mock_2 = vec!["key", "m2"];

        let result_received_1 = Sismember.run(buffer_vec_mock_1, &mut database_mock);
        let result_received_2 = Sismember.run(buffer_vec_mock_2, &mut database_mock);

        let excepted_1 = RInteger::encode(1);
        let excepted_2 = RInteger::encode(1);
        assert_eq!(excepted_1, result_received_1.unwrap());
        assert_eq!(excepted_2, result_received_2.unwrap());
    }

    #[test]
    fn test02_sismember_return_number_zero_if_member_is_not_contained_in_set() {
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key", "m_random"];

        let result_received = Sismember.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test03_sismember_return_number_zero_if_the_key_of_set_dont_exist_in_database() {
        // TODO: revisar nombres de tests... "in database" ...
        let mut set = HashSet::new();
        set.insert(String::from("m1"));
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::Set(set));
        let buffer_vec_mock = vec!["key_random", "m_random"];

        let result_received = Sismember.run(buffer_vec_mock, &mut database_mock);

        let excepted = RInteger::encode(0);
        assert_eq!(excepted, result_received.unwrap());
    }

    #[test]
    fn test04_sismember_return_error_wrongtype_if_execute_with_key_of_string() {
        let mut database_mock = Database::new();
        database_mock.insert(
            "keyOfString".to_string(),
            TypeSaved::String("value".to_string()),
        );
        let buffer_vec_mock = vec!["keyOfString", "value"];

        let result_received = Sismember.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test05_sismember_return_error_wrongtype_if_execute_with_key_of_list() {
        let mut database_mock = Database::new();
        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());
        new_list.push_back("value_other".to_string());
        database_mock.insert("keyOfList".to_string(), TypeSaved::List(new_list));

        let buffer_vec_mock = vec!["keyOfList", "value"];

        let result_received = Sismember.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::wrongtype();
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }

    #[test]
    fn test06_sismember_return_error_arguments_invalid_if_buffer_has_more_than_3_arguments() {
        let mut database_mock = Database::new();
        let buffer_vec_mock = vec!["arg1", "arg2", "arg3"];

        let result_received = Sismember.run(buffer_vec_mock, &mut database_mock);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let expected_message_redis = redis_messages::arguments_invalid_to("sismember");
        let expected_result =
            ("-".to_owned() + &expected_message_redis.get_message_complete() + "\r\n").to_string();
        assert_eq!(expected_result, result_received_encoded);
    }
}
