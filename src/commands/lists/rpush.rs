use crate::commands::Runnable;
use crate::database::Database;
use crate::native_types::error::ErrorStruct;

use super::fill_list_from_bottom;
use super::push_at;

pub struct RPush;

impl Runnable<Database> for RPush {
    fn run(&self, buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        push_at(buffer, database, fill_list_from_bottom)
    }
}

#[cfg(test)]
pub mod test_rpush {
    use crate::commands::create_notifier;

    use std::collections::VecDeque;

    use crate::{database::TypeSaved, vec_strings};

    use super::*;

    #[test]
    fn test01_rpush_values_on_an_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Database::new(notifier);
        let mut new_list = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "with", "new", "values"];
        let encode = RPush.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":7\r\n".to_string());
        match data.get_mut("key").unwrap() {
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
    fn test02_rpush_values_on_a_non_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Database::new(notifier);
        let buffer = vec_strings!["key", "this", "is", "a", "list"];
        let encode = RPush.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":4\r\n".to_string());
        match data.get_mut("key").unwrap() {
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
