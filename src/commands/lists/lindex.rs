use std::collections::VecDeque;

use crate::{
    commands::lists::{check_empty_2, check_not_empty},
    database::{Database, TypeSaved},
    native_types::bulk_string::RBulkString,
};
use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, redis_type::RedisType},
};

pub struct LIndex;

impl Runnable<Database> for LIndex {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
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

    use crate::vec_strings;

    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test01_lindex_positive_from_an_existing_list() {
        let mut data = Database::new();
        let mut new_list = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "2"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$1\r\na\r\n".to_string());
    }

    #[test]
    fn test02_lindex_negative_from_an_existing_list() {
        let mut data = Database::new();
        let mut new_list = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-1"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$4\r\nlist\r\n".to_string());
    }

    #[test]
    fn test03_lindex_from_a_non_existing_list() {
        let mut data = Database::new();
        let buffer = vec_strings!["key", "4"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$-1\r\n".to_string());
        assert_eq!(data.get("key"), None);
    }

    #[test]
    fn test04_lindex_out_of_index_from_an_existing_list() {
        let mut data = Database::new();
        let mut new_list = VecDeque::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));
        let buffer = vec_strings!["key", "6"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$-1\r\n".to_string());
    }
}
