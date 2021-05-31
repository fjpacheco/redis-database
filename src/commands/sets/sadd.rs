use std::collections::HashSet;

use crate::{
    commands::{
        check_empty_and_name_command,
        database_mock::{Database, TypeSaved},
    },
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Sadd;

impl Sadd {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let key = buffer_vec[1];
        let mut count_insert = 0;

        match database.get_mut(key) {
            Some(item) => match item {
                TypeSaved::Set(item) => {
                    count_insert = insert_in_set(&buffer_vec, item);
                    Ok(())
                }
                _ => {
                    let message_error = redis_messages::wrongtype();
                    Err(ErrorStruct::new(
                        message_error.get_prefix(),
                        message_error.get_message(),
                    ))
                }
            },
            None => {
                let mut set: HashSet<String> = HashSet::new();
                count_insert = insert_in_set(&buffer_vec, &mut set);
                database.insert(key.to_string(), TypeSaved::Set(set));
                Ok(())
            }
        }?;

        Ok(RInteger::encode(count_insert as isize))
    }
}
// Insert the "members" into the received set, according to what is indicated by the vector buffer (for example: "sadd key member1 member2 ..")
// Returns the number of insertions new in the set (repeated ones is ignored)
fn insert_in_set(buffer_vec: &[&str], item: &mut HashSet<String>) -> usize {
    buffer_vec
        .iter()
        .skip(2)
        .map(|member| item.insert(member.to_string()))
        .filter(|x| *x)
        .count()
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty_and_name_command(&buffer_vec, "sadd")?;

    if buffer_vec.len() < 3 {
        let error_message = redis_messages::wrong_number_args_for("sadd");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_sadd_function {

    use crate::{
        commands::{
            database_mock::{Database, TypeSaved},
            sets::sadd::Sadd,
        },
        native_types::{RInteger, RedisType},
    };

    #[test]
    fn test01_sadd_insert_and_return_amount_insertions() {
        let buffer_vec_mock = vec!["sadd", "key", "member1", "member2"];
        let mut database_mock = Database::new();

        let result_received = Sadd::run(buffer_vec_mock, &mut database_mock);
        let amount_received = result_received.unwrap();

        let expected = RInteger::encode(2);
        assert_eq!(expected, amount_received);
    }

    #[test]
    fn test02_sadd_does_not_insert_repeated_elements() {
        let buffer_vec_mock = vec![
            "sadd", "key", "member2", "member1", "member1", "member3", "member2", "member1",
            "member1", "member3",
        ];
        let mut database_mock = Database::new();

        let result_received = Sadd::run(buffer_vec_mock, &mut database_mock);
        let amount_received = result_received.unwrap();

        let expected = RInteger::encode(3);
        assert_eq!(expected, amount_received);
    }

    #[test]
    fn test03_sadd_does_not_insert_elements_over_an_existing_key_string() {
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock = vec![
            "sadd", "key", "member2", "member1", "member1", "member3", "member2", "member1",
            "member1", "member3",
        ];

        let result_received = Sadd::run(buffer_vec_mock, &mut database_mock);

        assert!(result_received.is_err())
    }

    #[test]
    fn test04_sadd_does_not_insert_elements_over_an_existing_key_list() {
        let mut database_mock = Database::new();
        database_mock.insert(
            "key".to_string(),
            TypeSaved::List(vec!["valueOfList".to_string()]),
        );
        let buffer_vec_mock = vec![
            "sadd", "key", "member2", "member1", "member1", "member3", "member2", "member1",
            "member1", "member3",
        ];

        let result_received = Sadd::run(buffer_vec_mock, &mut database_mock);

        assert!(result_received.is_err())
    }
}
