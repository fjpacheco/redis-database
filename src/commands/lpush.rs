use super::database_mock::{fill_list_from_top, push_at};
use crate::commands::database_mock::DatabaseMock;
use crate::native_types::error::ErrorStruct;

pub struct LPush;

impl LPush {
    pub fn run(buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        push_at(0, buffer, database, fill_list_from_top)
    }
}

#[cfg(test)]
pub mod test_lpush {

    use super::*;
    use crate::commands::database_mock::TypeSaved;

    #[test]
    fn test01_lpush_values_on_an_existing_list() {
        let mut data = DatabaseMock::new();
        let new_list: Vec<String> =
            vec!["with".to_string(), "new".to_string(), "values".to_string()];
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "list", "a", "is", "this"];
        let encode = LPush::run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":7\r\n".to_string());
        match data.get("key").unwrap() {
            TypeSaved::List(list) => {
                assert_eq!(&list[0], "this");
                assert_eq!(&list[1], "is");
                assert_eq!(&list[2], "a");
                assert_eq!(&list[3], "list");
                assert_eq!(&list[4], "with");
                assert_eq!(&list[5], "new");
                assert_eq!(&list[6], "values");
            }
            _ => {}
        }
    }

    #[test]
    fn test02_lpush_values_on_a_non_existing_list() {
        let mut data = DatabaseMock::new();
        let buffer: Vec<&str> = vec!["key", "this", "is", "a", "list"];
        let encode = LPush::run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":4\r\n".to_string());
        match data.get("key").unwrap() {
            TypeSaved::List(list) => {
                assert_eq!(&list[0], "list");
                assert_eq!(&list[1], "a");
                assert_eq!(&list[2], "is");
                assert_eq!(&list[3], "this");
            }
            _ => {}
        }
    }
}
