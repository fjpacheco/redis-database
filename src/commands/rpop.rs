use super::database_mock::{pop_at, remove_values_from_bottom};
use crate::commands::database_mock::DatabaseMock;
use crate::native_types::error::ErrorStruct;
pub struct RPop;

impl RPop {
    pub fn run(buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        pop_at(buffer, database, remove_values_from_bottom)
    }
}

#[cfg(test)]
pub mod test_rpop {

    use super::*;
    use crate::commands::database_mock::TypeSaved;
    use std::collections::LinkedList;
    #[test]
    fn test01_lpop_one_value_from_an_existing_list() {
        let mut data = DatabaseMock::new();
        let mut new_list: LinkedList<String> = LinkedList::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key"];
        let encode = RPop::run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$4\r\nlist\r\n".to_string());
        match data.get("key").unwrap() {
            TypeSaved::List(list) => {
                let mut list_iter = list.iter();
                assert_eq!(list_iter.next(), Some(&"this".to_string()));
                assert_eq!(list_iter.next(), Some(&"is".to_string()));
                assert_eq!(list_iter.next(), Some(&"a".to_string()));
                assert_eq!(list_iter.next(), None);
            }
            _ => {}
        }
    }

    #[test]
    fn test02_lpop_many_values_from_an_existing_list() {
        let mut data = DatabaseMock::new();
        let mut new_list: LinkedList<String> = LinkedList::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));
        let buffer = vec!["key", "3"];
        let encode = RPop::run(buffer, &mut data);
        assert_eq!(
            encode.unwrap(),
            "*3\r\n$4\r\nlist\r\n$1\r\na\r\n$2\r\nis\r\n".to_string()
        );
        match data.get("key").unwrap() {
            TypeSaved::List(list) => {
                let mut list_iter = list.iter();
                assert_eq!(list_iter.next(), Some(&"this".to_string()));
                assert_eq!(list_iter.next(), None);
            }
            _ => {}
        }
    }

    #[test]
    fn test03_lpop_value_from_a_non_existing_list() {
        let mut data = DatabaseMock::new();
        let buffer = vec!["key"];
        let encode = RPop::run(buffer, &mut data);
        assert_eq!(encode.unwrap(), "$-1\r\n".to_string());
        assert_eq!(data.get("key"), None);
    }

    #[test]
    fn test04_lpop_with_no_key() {
        let mut data = DatabaseMock::new();
        let buffer = vec![];
        match RPop::run(buffer, &mut data) {
            Ok(_encode) => {}
            Err(error) => assert_eq!(
                error.print_it(),
                "ERR wrong number of arguments for command".to_string()
            ),
        }
    }
}
