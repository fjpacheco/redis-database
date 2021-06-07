use std::collections::VecDeque;

use crate::commands::get_as_integer;
use crate::commands::lists::{check_empty, check_not_empty};
use crate::commands::Runnable;
use crate::database::Database;
use crate::database::TypeSaved;
use crate::native_types::RedisType;
use crate::native_types::{error::ErrorStruct, simple_string::RSimpleString};

pub struct Lset;

// Sets the list element at index to element.
//
// An error is returned for out of range indexes.

impl Runnable for Lset {
    fn run(&self, mut buffer: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_not_empty(&buffer)?;
        let key = String::from(buffer.remove(0));
        check_not_empty(&buffer)?;
        let index = get_as_integer(buffer.remove(0)).unwrap();
        check_not_empty(&buffer)?;
        let replacement = String::from(buffer.remove(0));
        check_empty(&buffer)?;

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

    use super::*;
    use std::collections::{vec_deque::Iter, VecDeque};

    #[test]
    fn test01_lset_list_with_one_element_positive_indexing() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "0", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        let mut iter = None;
        if let TypeSaved::List(values_list) = data.get_mut(&key_cpy).unwrap() {
            iter = Some(values_list.iter());
        }
        assert_eq!(
            iter.unwrap().next().unwrap().to_string(),
            "new_value".to_string()
        );
    }

    #[test]
    fn test02_lset_list_with_one_element_negative_indexing() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "-1", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        let mut iter = None;
        if let TypeSaved::List(values_list) = data.get_mut(&key_cpy).unwrap() {
            iter = Some(values_list.iter());
        }
        assert_eq!(
            iter.unwrap().next().unwrap().to_string(),
            "new_value".to_string()
        );
    }
    #[test]
    fn test03_lset_list_with_out_of_range_positive_index() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "1", "new_value"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR index out of range".to_string()
        );
    }

    #[test]
    fn test04_lset_list_with_out_of_range_negative_index() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "-2", "new_value"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR index out of range".to_string()
        );
    }

    #[test]
    fn test05_lset_list_non_existent_key() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        let key = "key1".to_string();
        data.insert(key, TypeSaved::List(new_list));

        // key2 was not inserted (key1 was)
        let buffer = vec!["key2", "-2", "new_value"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(error.unwrap_err().print_it(), "ERR no such key".to_string());
    }

    #[test]
    fn test06_lset_non_list() {
        let mut data = Database::new();
        // redis> SET mykey 10
        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec!["key", "1", "new_value"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
        );
    }

    #[test]
    fn test07_lset_list_with_many_elements_at_top() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "0", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        let mut iter = None;
        if let TypeSaved::List(values_list) = data.get_mut(&key_cpy).unwrap() {
            iter = Some(values_list.iter());
        }
        assert_eq!(
            iter.unwrap().next().unwrap().to_string(),
            "new_value".to_string()
        );
    }

    #[test]
    fn test08_lset_list_with_many_elements_at_middle() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "1", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        if let TypeSaved::List(values_list) = data.get_mut(&key_cpy).unwrap() {
            let mut iter: Iter<String> = values_list.iter();
            iter.next();
            assert_eq!(iter.next().unwrap().to_string(), "new_value".to_string());
        }
    }

    #[test]
    fn test09_lset_list_with_many_elements_at_bottom() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "2", "new_value"];
        let encoded = Lset.run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        if let TypeSaved::List(values_list) = data.get_mut(&key_cpy).unwrap() {
            let mut iter: Iter<String> = values_list.iter();
            iter.next();
            iter.next();
            assert_eq!(iter.next().unwrap().to_string(), "new_value".to_string());
        }
    }

    #[test]
    fn test11_lset_with_zero_arguments() {
        let mut data = Database::new();

        let new_list = VecDeque::new();

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec![];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR wrong number of arguments".to_string()
        );
    }

    #[test]
    fn test12_lset_with_wrong_number_of_arguments() {
        let mut data = Database::new();

        let new_list = VecDeque::new();

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "2", "new_value_1", "new_value_2"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR wrong number of arguments".to_string()
        );
    }

    #[test]
    fn test13_lset_with_wrong_number_of_arguments() {
        let mut data = Database::new();

        let new_list = VecDeque::new();

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "2"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR wrong number of arguments".to_string()
        );
    }

    #[test]
    fn test14_lset_with_wrong_number_of_arguments() {
        let mut data = Database::new();

        let new_list = VecDeque::new();

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key"];
        let error = Lset.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR wrong number of arguments".to_string()
        );
    }
}