use crate::{
    commands::database_mock::{fill_list_from_bottom, pushx_at, Database},
    native_types::error::ErrorStruct,
};

pub struct RPushx;

impl RPushx {
    pub fn run(buffer: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        pushx_at(buffer, database, fill_list_from_bottom)
    }
}

#[cfg(test)]
pub mod test_rpushx {

    use std::collections::LinkedList;

    use super::*;
    use crate::commands::database_mock::{Database, TypeSaved};

    #[test]
    fn test01_rpushx_values_on_an_existing_list() {
        let mut data = Database::new();
        let mut new_list = LinkedList::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "with", "new", "values"];
        let encode = RPushx::run(buffer, &mut data);
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
    fn test02_rpushx_values_on_a_non_existing_list() {
        let mut data = Database::new();
        let buffer: Vec<&str> = vec!["key", "this", "is", "a", "list"];
        let error = RPushx::run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR no list found with entered key".to_string()
        );
    }
}
