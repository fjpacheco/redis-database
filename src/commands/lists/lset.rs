use crate::commands::get_as_integer;
use crate::commands::lists::{check_empty, check_not_empty};
use crate::commands::Runnable;
use crate::database::Database;
use crate::database::TypeSaved;
use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::RedisType;
use crate::native_types::{error::ErrorStruct, simple_string::RSimpleString};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct Lset;

// Sets the list element at index to element.
//
// An error is returned for out of range indexes.

impl Runnable<Arc<Mutex<Database>>> for Lset {
    /// Sets the list element at index to element. An error is returned for out of range indexes.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: "OK".
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a list.
    /// * Buffer [Vec]<[String]> is received empty, or received with an amount of elements different than 3.
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
        check_empty(&buffer, "lset")?;
        let key = buffer.remove(0);
        check_empty(&buffer, "lset")?;
        let index = get_as_integer(&buffer.remove(0)).unwrap();
        check_empty(&buffer, "lset")?;
        let replacement = buffer.remove(0);
        check_not_empty(&buffer)?;

        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::List(values_list) => replace_element_at(index, replacement, values_list),
                _ => Err(ErrorStruct::new(
                    String::from("WRONGTYPE"),
                    String::from("Operation against a key holding the wrong kind of value"),
                )),
            }
        } else {
            Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("no such key"),
            ))
        }
    }
}

// Executes the replace of the element at the given index in the VecDeque.
// Returns a result which can be an "OK" encoded as Simple String or an
// ErrorStruct in case of error, when the index is out of range.
pub fn replace_element_at(
    mut index: isize,
    replacement: String,
    values_list: &mut VecDeque<String>,
) -> Result<String, ErrorStruct> {
    let len = values_list.len() as isize;
    if index < 0 {
        index += len;
    }
    if (index >= len) || (index < 0) {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("index out of range"),
        ))
    } else {
        for (i, element) in values_list.iter_mut().enumerate() {
            if i == index as usize {
                // at this point index couldn't be negative
                *element = replacement;
                break;
            }
        }
        Ok(RSimpleString::encode("OK".to_string()))
    }
}

#[cfg(test)]
pub mod test_lset {
    use crate::commands::create_notifier;

    use crate::vec_strings;

    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_01_lset_list_with_one_element_positive_indexing() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();

        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "0", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        if let TypeSaved::List(values_list) = data.lock().unwrap().get_mut(&key_cpy).unwrap() {
            let iter = Some(values_list.iter());
            assert_eq!(
                iter.unwrap().next().unwrap().to_string(),
                "new_value".to_string()
            );
        } else {
            panic!();
        };
    }

    #[test]
    fn test_02_lset_list_with_one_element_negative_indexing() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();

        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-1", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        if let TypeSaved::List(values_list) = data.lock().unwrap().get_mut(&key_cpy).unwrap() {
            let iter = Some(values_list.iter());
            assert_eq!(
                iter.unwrap().next().unwrap().to_string(),
                "new_value".to_string()
            );
        } else {
            panic!();
        };
    }
    #[test]
    fn test_03_lset_list_with_out_of_range_positive_index() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "1", "new_value"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR index out of range".to_string()
        );
    }

    #[test]
    fn test_04_lset_list_with_out_of_range_negative_index() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-2", "new_value"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR index out of range".to_string()
        );
    }

    #[test]
    fn test_05_lset_list_non_existent_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key1".to_string();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        // key2 was not inserted (key1 was)
        let buffer = vec_strings!["key2", "-2", "new_value"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(error.unwrap_err().print_it(), "ERR no such key".to_string());
    }

    #[test]
    fn test_06_lset_non_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key", "1", "new_value"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
        );
    }

    #[test]
    fn test_07_lset_list_with_many_elements_at_top() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();

        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "0", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        if let TypeSaved::List(values_list) = data.lock().unwrap().get_mut(&key_cpy).unwrap() {
            let iter = Some(values_list.iter());
            assert_eq!(
                iter.unwrap().next().unwrap().to_string(),
                "new_value".to_string()
            );
        } else {
            panic!();
        };
    }

    #[test]
    fn test_08_lset_list_with_many_elements_at_middle() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();

        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "1", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        if let TypeSaved::List(values_list) = data.lock().unwrap().get_mut(&key_cpy).unwrap() {
            let mut iter = values_list.iter();
            iter.next();
            assert_eq!(iter.next().unwrap().to_string(), "new_value".to_string());
        } else {
            panic!();
        };
    }

    #[test]
    fn test_09_lset_list_with_many_elements_at_bottom() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();

        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "2", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        if let TypeSaved::List(values_list) = data.lock().unwrap().get_mut(&key_cpy).unwrap() {
            let mut iter = values_list.iter();
            iter.next();
            iter.next();
            assert_eq!(iter.next().unwrap().to_string(), "new_value".to_string());
        } else {
            panic!();
        };
    }

    #[test]
    fn test_11_lset_with_zero_arguments() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let new_list = VecDeque::new();

        let key = "key".to_string();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings![];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR wrong number of arguments for 'lset' command".to_string()
        );
    }

    #[test]
    fn test_12_lset_with_wrong_number_of_arguments() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let new_list = VecDeque::new();

        let key = "key".to_string();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "2", "new_value_1", "new_value_2"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR wrong number of arguments".to_string()
        );
    }

    #[test]
    fn test_13_lset_with_wrong_number_of_arguments() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let new_list = VecDeque::new();

        let key = "key".to_string();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "2"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR wrong number of arguments for 'lset' command".to_string()
        );
    }

    #[test]
    fn test_14_lset_with_wrong_number_of_arguments() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let new_list = VecDeque::new();

        let key = "key".to_string();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        let buffer = vec_strings!["key"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR wrong number of arguments for 'lset' command".to_string()
        );
    }
}
