use super::database_mock::{fill_list_from_bottom, push_at};
use crate::commands::database_mock::DatabaseMock;
use crate::native_types::error::ErrorStruct;

pub struct RPush;

impl RPush {
    pub fn run(buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        push_at(buffer, database, fill_list_from_bottom)
    }
}

#[cfg(test)]
pub mod test_rpush {

    use std::collections::LinkedList;

    use super::RPush;
    use crate::commands::database_mock::{DatabaseMock, TypeSaved};

    #[test]
    fn test01_rpush_values_on_an_existing_list() {
        let mut data = DatabaseMock::new();
        let mut new_list = LinkedList::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "with", "new", "values"];
        let encode = RPush::run(buffer, &mut data);
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
        let mut data = DatabaseMock::new();
        let buffer: Vec<&str> = vec!["key", "this", "is", "a", "list"];
        let encode = RPush::run(buffer, &mut data);
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
