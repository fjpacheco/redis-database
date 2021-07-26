use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::lists::{check_empty_2, check_not_empty},
    database::{Database, TypeSaved},
    native_types::bulk_string::RBulkString,
};
use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, redis_type::RedisType},
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct LIndex;

impl Runnable<Arc<Mutex<Database>>> for LIndex {
    /// Returns the element at index index in the list stored at key. The index is zero-based,
    /// so 0 means the first element, 1 the second element and so on. Negative indices can be
    /// used to designate elements starting at the tail of the list. Here, -1 means the last element, -2 means the penultimate and so forth.
    /// Time complexity: O(N) where N is the number of elements to traverse to get to the element at index. This makes asking for the first or the last element of the list O(1).
    /// When the value at key is not a list, an error is returned.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger]: the requested element, or nil when index is out of range.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a list.
    /// * Buffer [Vec]<[String]> is received empty, or received with less or more than 2 elements.
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
        check_not_empty(&buffer)?;
        let index = parse_index(&mut buffer)?;
        check_empty_2(&buffer)?;

        if let Some(typesaved) = database.get(&key) {
            match typesaved {
                TypeSaved::List(list_of_values) => Ok(get_from_index(index, list_of_values)),
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from lists"),
                )),
            }
        } else {
            Ok(RBulkString::encode("(nil)".to_string()))
        }
    }
}

// Obtains an integer (isize) from a String received at a vector and returns it.
// If the String was not a parsable integer, returns error.
fn parse_index(buffer: &mut Vec<String>) -> Result<isize, ErrorStruct> {
    if let Some(value) = buffer.pop() {
        if let Ok(index) = value.parse::<isize>() {
            Ok(index)
        } else {
            Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("value is not an integer or out of range"),
            ))
        }
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("wrong number of arguments"),
        ))
    }
}

// Looks for the element at index in the VecDeque list. If there is an element at the index
// it returns it encoded as Bulk String, if there's not, it returns "(nil)" also encoded as
// Bulk String. This function accepts negative index values.
fn get_from_index(mut index: isize, list: &VecDeque<String>) -> String {
    if index < 0 {
        index += list.len() as isize;
    }
    if let Some(string) = list.get(index as usize) {
        RBulkString::encode(String::from(string))
    } else {
        RBulkString::encode("(nil)".to_string())
    }
}

#[cfg(test)]
pub mod test_lpush {
    use crate::commands::create_notifier;

    use crate::vec_strings;

    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_01_lindex_positive_from_an_existing_list() {
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

        let buffer = vec_strings!["key", "2"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$1\r\na\r\n".to_string());
    }

    #[test]
    fn test_02_lindex_negative_from_an_existing_list() {
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

        let buffer = vec_strings!["key", "-1"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$4\r\nlist\r\n".to_string());
    }

    #[test]
    fn test_03_lindex_from_a_non_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["key", "4"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$-1\r\n".to_string());
        assert_eq!(data.lock().unwrap().get("key"), None);
    }

    #[test]
    fn test_04_lindex_out_of_index_from_an_existing_list() {
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
        let buffer = vec_strings!["key", "6"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$-1\r\n".to_string());
    }
}
