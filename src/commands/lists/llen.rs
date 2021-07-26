use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::lists::{check_empty_2, check_not_empty},
    commands::Runnable,
    database::{Database, TypeSaved},
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};
use std::sync::{Arc, Mutex};
pub struct Llen;

impl Runnable<Arc<Mutex<Database>>> for Llen {
    /// Returns the length of the list stored at key. If key does not exist, it is interpreted as an
    /// empty list and 0 is returned. An error is returned when the value stored at key is not a list.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger]: the length of the list at key.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a list.
    /// * Buffer [Vec]<[String]> is received empty, or received with 2 or more elements.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        mut buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        check_not_empty(&buffer)?;
        let key = buffer.remove(0);
        check_empty_2(&buffer)?;
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::List(list_of_values) => {
                    Ok(RInteger::encode(list_of_values.len() as isize))
                }
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("value stored at key is not a list"),
                )),
            }
        } else {
            // Key does not exist, interpreted as empty list
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
pub mod test_llen {
    use crate::commands::create_notifier;

    use crate::vec_strings;

    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_01_llen_an_existing_list_of_one_element() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key"];
        let encode = Llen.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":1\r\n".to_string());
    }

    #[test]
    fn test_02_llen_an_existing_list_of_many_elements() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key"];
        let encode = Llen.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":4\r\n".to_string());
    }

    #[test]
    fn test_03_llen_to_key_storing_non_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key"];
        let error = Llen.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value stored at key is not a list".to_string()
        );
    }
}
