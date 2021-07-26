use super::fill_list_from_bottom;
use super::push_at;
use crate::commands::Runnable;
use crate::database::Database;
use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};

pub struct RPush;

impl Runnable<Arc<Mutex<Database>>> for RPush {
    /// Insert all the specified values at the tail of the list stored at key. If key does
    /// not exist, it is created as empty list before performing the push operation. When
    /// key holds a value that is not a list, an error is returned.
    /// It is possible to push multiple elements using a single command call just specifying
    /// multiple arguments at the end of the command. Elements are inserted one after the
    /// other to the tail of the list, from the leftmost element to the rightmost element.
    /// So for instance the command RPUSH mylist a b c will result into a list containing a
    /// as first element, b as second element and c as third element.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger]: the length of the list after the push operation.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a list.
    /// * Buffer [Vec]<[String]> is received empty, or received less than 2 elements.
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
        push_at(buffer, &mut database, fill_list_from_bottom)
    }
}

#[cfg(test)]
pub mod test_rpush {
    use crate::commands::create_notifier;

    use std::collections::VecDeque;

    use crate::{database::TypeSaved, vec_strings};

    use super::*;

    #[test]
    fn test_01_rpush_values_on_an_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();

        let mut new_list = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());

        let mut db = Database::new(notifier);
        db.insert("key".to_string(), TypeSaved::List(new_list));
        let mut data = Arc::new(Mutex::new(db));

        let buffer = vec_strings!["key", "with", "new", "values"];
        let encode = RPush.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":7\r\n".to_string());
        let mut b = data.lock().unwrap();
        match b.get_mut("key").unwrap() {
            TypeSaved::List(list) => {
                assert_eq!(list.pop_front().unwrap(), "this");
                assert_eq!(list.pop_front().unwrap(), "is");
                assert_eq!(list.pop_front().unwrap(), "a");
                assert_eq!(list.pop_front().unwrap(), "list");
                assert_eq!(list.pop_front().unwrap(), "with");
                assert_eq!(list.pop_front().unwrap(), "new");
                assert_eq!(list.pop_front().unwrap(), "values");
            }
            _ => {}
        }
    }

    #[test]
    fn test_02_rpush_values_on_a_non_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let c_data = Arc::clone(&data);

        let buffer = vec_strings!["key", "this", "is", "a", "list"];
        let encode = RPush.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":4\r\n".to_string());
        let mut c_db = c_data.lock().unwrap();

        match c_db.get_mut("key").unwrap() {
            TypeSaved::List(list) => {
                assert_eq!(list.pop_front().unwrap(), "this");
                assert_eq!(list.pop_front().unwrap(), "is");
                assert_eq!(list.pop_front().unwrap(), "a");
                assert_eq!(list.pop_front().unwrap(), "list");
            }
            _ => {}
        }
    }
}
