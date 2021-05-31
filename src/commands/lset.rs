use std::collections::LinkedList;

use crate::native_types::{error::ErrorStruct, simple_string::RSimpleString};
use crate::{commands::database_mock::DatabaseMock, native_types::redis_type::RedisType};

use super::database_mock::{get_as_integer, TypeSaved};

pub struct Lset;

// Sets the list element at index to element.
//
// An error is returned for out of range indexes.

impl Lset {
    pub fn run(mut buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        let key = String::from(buffer.remove(0));
        let replacement = String::from(buffer.remove(0));
        let index = get_as_integer(buffer.pop().unwrap()).unwrap();
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
    values_list: &mut LinkedList<String>,
) -> Result<String, ErrorStruct> {
    let len = values_list.len() as isize;
    if index < 0 {
        index += len;
    }
    if (index > 0 && index >= len) || (index < 0) {
        // index > 0 is redundant
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("index out of range"),
        ))
    } else {
        let mut i = 0;
        for element in values_list.iter_mut() {
            if i == index {
                *element = replacement;
                break;
            }
            i += 1;
        }
        Ok(RSimpleString::encode("OK".to_string()))
    }
}

#[cfg(test)]
pub mod test_lset {

    use crate::commands::database_mock::{DatabaseMock, TypeSaved};
    use std::collections::{linked_list::Iter, LinkedList};

    use super::Lset;

    #[test]
    fn test01_lset_list_with_one_element_positive_indexing() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "new_value", "0"];
        let encoded = Lset::run(buffer, &mut data);

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
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "new_value", "-1"];
        let encoded = Lset::run(buffer, &mut data);

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
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "new_value", "1"];
        let error = Lset::run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR index out of range".to_string()
        );
    }

    #[test]
    fn test04_lset_list_with_out_of_range_negative_index() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value".to_string());

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "new_value", "-2"];
        let error = Lset::run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR index out of range".to_string()
        );
    }

    #[test]
    fn test05_lset_list_non_existent_key() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value".to_string());

        let key = "key1".to_string();
        data.insert(key, TypeSaved::List(new_list));

        // key2 was not inserted (key1 was)
        let buffer = vec!["key2", "new_value", "-2"];
        let error = Lset::run(buffer, &mut data);
        assert_eq!(error.unwrap_err().print_it(), "ERR no such key".to_string());
    }

    #[test]
    fn test06_lset_non_list() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey 10
        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec!["key", "new_value", "1"];
        let error = Lset::run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
        );
    }

    #[test]
    fn test07_lset_list_with_many_elements_at_top() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "new_value", "0"];
        let encoded = Lset::run(buffer, &mut data);

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
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "new_value", "1"];
        let encoded = Lset::run(buffer, &mut data);

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
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        let key = "key".to_string();
        let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        let buffer = vec!["key", "new_value", "2"];
        let encoded = Lset::run(buffer, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded.unwrap(), "+OK\r\n".to_string());

        if let TypeSaved::List(values_list) = data.get_mut(&key_cpy).unwrap() {
            let mut iter: Iter<String> = values_list.iter();
            iter.next();
            iter.next();
            assert_eq!(iter.next().unwrap().to_string(), "new_value".to_string());
        }
    }
}
