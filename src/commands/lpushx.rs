use super::database_mock::{fill_list_from_top, pushx_at};
use crate::commands::database_mock::DatabaseMock;
use crate::native_types::error::ErrorStruct;

pub struct LPushx;

impl LPushx {
    pub fn run(buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        pushx_at(buffer, database, fill_list_from_top)
    }
}

#[cfg(test)]
pub mod test_lpushx {

    use std::collections::LinkedList;

    use super::*;
    use crate::commands::database_mock::TypeSaved;

    #[test]
    fn test01_lpushx_values_on_an_existing_list() {
        let mut data = DatabaseMock::new();
        let mut new_list = LinkedList::new();
        new_list.push_back("with".to_string());
        new_list.push_back("new".to_string());
        new_list.push_back("values".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "list", "a", "is", "this"];
        let encode = LPushx::run(buffer, &mut data);
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
    fn test02_lpushx_values_on_a_non_existing_list() {
        let mut data = DatabaseMock::new();
        let buffer: Vec<&str> = vec!["key", "this", "is", "a", "list"];
        let error = LPushx::run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR no list found with entered key".to_string()
        );
    }
}
