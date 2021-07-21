use super::fill_list_from_top;
use super::push_at;
use crate::commands::Runnable;
use crate::database::Database;
use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};

pub struct LPush;

impl Runnable<Arc<Mutex<Database>>> for LPush {
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
        push_at(buffer, &mut database, fill_list_from_top)
    }
}

#[cfg(test)]
pub mod test_lpush {
    use crate::commands::create_notifier;

    use std::collections::VecDeque;

    use crate::{database::TypeSaved, vec_strings};

    use super::*;

    #[test]
    fn test01_lpush_values_on_an_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut new_list = VecDeque::new();
        new_list.push_back("with".to_string());
        new_list.push_back("new".to_string());
        new_list.push_back("values".to_string());

        let mut db = Database::new(notifier);
        db.insert("key".to_string(), TypeSaved::List(new_list));
        let mut data = Arc::new(Mutex::new(db));

        let buffer = vec_strings!["key", "list", "a", "is", "this"];
        let encode = LPush.run(buffer, &mut data);
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
    fn test02_lpush_values_on_a_non_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let c_data = Arc::clone(&data);

        let buffer = vec_strings!["key", "this", "is", "a", "list"];
        let encode = LPush.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":4\r\n".to_string());
        let mut c_db = c_data.lock().unwrap();

        match c_db.get_mut("key").unwrap() {
            TypeSaved::List(list) => {
                assert_eq!(list.pop_front().unwrap(), "list");
                assert_eq!(list.pop_front().unwrap(), "a");
                assert_eq!(list.pop_front().unwrap(), "is");
                assert_eq!(list.pop_front().unwrap(), "this");
            }
            _ => {}
        }
    }
}
