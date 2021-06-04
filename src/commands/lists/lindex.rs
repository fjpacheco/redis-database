use crate::commands::Runnable;
use crate::native_types::bulk_string::RBulkString;
use crate::native_types::error::ErrorStruct;
use crate::native_types::redis_type::RedisType;

use crate::database::{Database, TypeSaved};
use super::{parse_index, get_from_index};

pub struct LIndex;

impl Runnable for LIndex {
    fn run(&self, mut buffer: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        let key = String::from(buffer.remove(0));
        let index = parse_index(&mut buffer)?;

        if !buffer.is_empty() {
            return Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("wrong number of arguments for 'lindex' command"),
            ));
        }

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

#[cfg(test)]
pub mod test_lpush {

    use super::*;

    #[test]
    fn test01_lindex_positive_from_an_existing_list() {
        let mut data = Database::new();
        let new_list: Vec<String> = vec![
            "this".to_string(),
            "is".to_string(),
            "a".to_string(),
            "list".to_string(),
        ];
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "2"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$1\r\na\r\n".to_string());
    }

    #[test]
    fn test02_lindex_negative_from_an_existing_list() {
        let mut data = Database::new();
        let new_list: Vec<String> = vec![
            "this".to_string(),
            "is".to_string(),
            "a".to_string(),
            "list".to_string(),
        ];
        data.insert("key".to_string(), TypeSaved::List(new_list));
        let buffer = vec!["key", "-1"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$4\r\nlist\r\n".to_string());
    }

    #[test]
    fn test03_lindex_from_a_non_existing_list() {
        let mut data = Database::new();
        let buffer = vec!["key", "4"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$-1\r\n".to_string());
        assert_eq!(data.get("key"), None);
    }

    #[test]
    fn test04_lindex_out_of_index_from_an_existing_list() {
        let mut data = Database::new();
        let new_list: Vec<String> = vec![
            "this".to_string(),
            "is".to_string(),
            "a".to_string(),
            "list".to_string(),
        ];
        data.insert("key".to_string(), TypeSaved::List(new_list));
        let buffer = vec!["key", "6"];
        let encode = LIndex.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$-1\r\n".to_string());
    }
}
