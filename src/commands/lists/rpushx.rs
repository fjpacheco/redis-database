use crate::{
    commands::Runnable, database::Database, messages::redis_messages,
    native_types::error::ErrorStruct,
};

use super::{fill_list_from_bottom, pushx_at};
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};
pub struct RPushx;

impl Runnable<Arc<Mutex<Database>>> for RPushx {
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
        pushx_at(buffer, &mut database, fill_list_from_bottom)
    }
}

#[cfg(test)]
pub mod test_rpushx {
    use crate::commands::create_notifier;

    use super::*;
    use crate::{database::TypeSaved, vec_strings};
    use std::collections::VecDeque;

    #[test]
    fn test01_rpushx_values_on_an_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let c_data = Arc::clone(&data);

        let mut new_list = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "with", "new", "values"];
        let encode = RPushx.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":7\r\n".to_string());
        let mut c_db = c_data.lock().unwrap();

        match c_db.get_mut("key").unwrap() {
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
    fn test02_rpushx_values_on_a_non_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["key", "this", "is", "a", "list"];
        let error = RPushx.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR no list found with entered key".to_string()
        );
    }
}
