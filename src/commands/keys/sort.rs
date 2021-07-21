use std::collections::{HashSet, VecDeque};

use super::{no_more_values, pop_value};
use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::Runnable,
    database::TypeSaved,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::RArray,
    native_types::{RBulkString, RedisType},
    Database,
};
use std::sync::{Arc, Mutex};
pub struct Sort;

impl Runnable<Arc<Mutex<Database>>> for Sort {
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
        let key = pop_value(&mut buffer, "Sort")?;
        no_more_values(&buffer, "Sort")?;

        if let Some(type_saved) = database.get(&key) {
            let sorted: Vec<String> = match type_saved {
                TypeSaved::String(string) => vec![String::from(string)],
                TypeSaved::List(list) => sort_list(list),
                TypeSaved::Set(set) => sort_set(set),
            };
            Ok(RArray::encode(sorted))
        } else {
            Ok(RBulkString::encode(String::from("(nil)")))
        }
    }
}

fn sort_list(list: &VecDeque<String>) -> Vec<String> {
    let mut sorted = list.iter().map(String::from).collect::<Vec<String>>();
    sorted.sort();
    sorted
}

fn sort_set(set: &HashSet<String>) -> Vec<String> {
    let mut sorted = set.iter().map(String::from).collect::<Vec<String>>();
    sorted.sort();
    sorted
}

#[cfg(test)]
pub mod test_llen {
    use crate::commands::create_notifier;

    use crate::commands::lists::lpush::LPush;

    use super::*;

    #[test]
    fn test01_sorting_a_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut db = Arc::new(Mutex::new(Database::new(notifier)));
        let _ = LPush.run(
            vec![
                "key".to_string(),
                "w".to_string(),
                "a".to_string(),
                "s".to_string(),
                "d".to_string(),
            ],
            &mut db,
        );
        let sorted = Sort.run(vec!["key".to_string()], &mut db);
        assert_eq!(
            &sorted.unwrap(),
            "*4\r\n$1\r\na\r\n$1\r\nd\r\n$1\r\ns\r\n$1\r\nw\r\n"
        );
    }
}
