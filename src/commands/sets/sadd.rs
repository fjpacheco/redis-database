use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
pub struct Sadd;

impl Runnable<Arc<Mutex<Database>>> for Sadd {
    /// Add the specified members to the set stored at **key**. Specified members that are already a member of this set are ignored.
    /// If **key** does not exist, a new set is created before adding the specified members.
    /// An error is returned when the value stored at key is not a set
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): the number of elements that were added to the set, not including all the elements already present in the set.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a set.
    /// * Buffer [Vec]<[String]> is received empty, or received with only one element.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        check_error_cases(&buffer)?;

        let key = &buffer[0];

        match database.get_mut(key) {
            Some(item) => match item {
                TypeSaved::Set(item) => {
                    let count_insert = insert_in_set(&buffer, item);
                    Ok(RInteger::encode(count_insert as isize))
                }
                _ => {
                    err_wrongtype!()
                }
            },
            None => {
                let mut set: HashSet<String> = HashSet::new();
                let count_insert = insert_in_set(&buffer, &mut set);
                database.insert(key.to_string(), TypeSaved::Set(set));
                Ok(RInteger::encode(count_insert as isize))
            }
        }
    }
}
// Insert the "members" into the received set, according to what is indicated by the vector buffer (for example: "sadd key member1 member2 ..")
// Returns the number of insertions new in the set (repeated ones is ignored)
fn insert_in_set(buffer: &[String], item: &mut HashSet<String>) -> usize {
    buffer
        .iter()
        .skip(1)
        .map(|member| item.insert(member.to_string()))
        .filter(|x| *x)
        .count()
}

fn check_error_cases(buffer: &[String]) -> Result<(), ErrorStruct> {
    check_empty(&buffer, "sadd")?;

    if buffer.len() < 2 {
        let error_message = redis_messages::arguments_invalid_to("sadd");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_sadd_function {
    use crate::commands::create_notifier;

    use crate::vec_strings;

    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_01_sadd_insert_and_return_amount_insertions() {
        let buffer_mock = vec_strings!["key", "member1", "member2"];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        let result_received = Sadd.run(buffer_mock, &mut database_mock);
        let amount_received = result_received.unwrap();

        let expected = RInteger::encode(2);
        assert_eq!(expected, amount_received);
    }

    #[test]
    fn test_02_sadd_does_not_insert_repeated_elements() {
        let buffer_mock = vec_strings![
            "key", "member2", "member1", "member1", "member3", "member2", "member1", "member1",
            "member3"
        ];
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));

        let result_received = Sadd.run(buffer_mock, &mut database_mock);
        let amount_received = result_received.unwrap();

        let expected = RInteger::encode(3);
        assert_eq!(expected, amount_received);
    }

    #[test]
    fn test_03_sadd_does_not_insert_elements_over_an_existing_key_string() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock = vec_strings![
            "key", "member2", "member1", "member1", "member3", "member2", "member1", "member1",
            "member3"
        ];

        let result_received = Sadd.run(buffer_mock, &mut database_mock);

        assert!(result_received.is_err())
    }

    #[test]
    fn test_04_sadd_does_not_insert_elements_over_an_existing_key_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database_mock = Arc::new(Mutex::new(Database::new(notifier)));
        let mut new_list = VecDeque::new();
        new_list.push_back("valueOfList".to_string());
        database_mock
            .lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer_mock = vec_strings![
            "key", "member2", "member1", "member1", "member3", "member2", "member1", "member1",
            "member3"
        ];

        let result_received = Sadd.run(buffer_mock, &mut database_mock);

        assert!(result_received.is_err())
    }
}
