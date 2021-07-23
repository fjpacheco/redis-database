use super::pop_at;
use super::remove_values_from_bottom;
use crate::commands::Runnable;
use crate::database::Database;
use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};
pub struct RPop;

impl Runnable<Arc<Mutex<Database>>> for RPop {
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
        pop_at(buffer, &mut database, remove_values_from_bottom)
    }
}

#[cfg(test)]
pub mod test_rpop {
    use crate::commands::create_notifier;

    use crate::{database::TypeSaved, vec_strings};

    use super::*;
    use std::collections::VecDeque;
    #[test]
    fn test_01_lpop_one_value_from_an_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();

        let mut new_list: VecDeque<String> = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());

        let mut db = Database::new(notifier);
        db.insert("key".to_string(), TypeSaved::List(new_list));
        let mut data = Arc::new(Mutex::new(db));

        let buffer = vec_strings!["key"];
        let encode = RPop.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$4\r\nlist\r\n".to_string());
        let mut b = data.lock().unwrap();
        match b.get("key").unwrap() {
            TypeSaved::List(list) => {
                let mut list_iter = list.iter();
                assert_eq!(list_iter.next(), Some(&"this".to_string()));
                assert_eq!(list_iter.next(), Some(&"is".to_string()));
                assert_eq!(list_iter.next(), Some(&"a".to_string()));
                assert_eq!(list_iter.next(), None);
            }
            _ => {}
        }
    }

    #[test]
    fn test_02_lpop_many_values_from_an_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();

        let mut new_list: VecDeque<String> = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        let mut db = Database::new(notifier);
        db.insert("key".to_string(), TypeSaved::List(new_list));
        let mut data = Arc::new(Mutex::new(db));

        let buffer = vec_strings!["key", "3"];
        let encode = RPop.run(buffer, &mut data);
        assert_eq!(
            encode.unwrap(),
            "*3\r\n$4\r\nlist\r\n$1\r\na\r\n$2\r\nis\r\n".to_string()
        );
        let mut b = data.lock().unwrap();
        match b.get("key").unwrap() {
            TypeSaved::List(list) => {
                let mut list_iter = list.iter();
                assert_eq!(list_iter.next(), Some(&"this".to_string()));
                assert_eq!(list_iter.next(), None);
            }
            _ => {}
        }
    }

    #[test]
    fn test_03_lpop_value_from_a_non_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["key"];
        let encode = RPop.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$-1\r\n".to_string());
        assert_eq!(data.lock().unwrap().get("key"), None);
    }

    #[test]
    fn test_04_lpop_with_no_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings![];
        match RPop.run(buffer, &mut data) {
            Ok(_encode) => {}
            Err(error) => assert_eq!(
                error.print_it(),
                "ERR wrong number of arguments".to_string()
            ),
        }
    }
}
