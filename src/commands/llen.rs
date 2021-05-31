use crate::native_types::error::ErrorStruct;
use crate::{
    commands::database_mock::DatabaseMock,
    native_types::{integer::RInteger, redis_type::RedisType},
};

use super::database_mock::TypeSaved;

pub struct Llen;

// Returns the length of the list stored at key. If key does not exist, it is
// interpreted as an empty list and 0 is returned. An error is returned when
// the value stored at key is not a list.

impl Llen {
    pub fn run(mut buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        let key = String::from(buffer.remove(0));
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::List(list_of_values) => {
                    Ok(RInteger::encode(list_of_values.len() as isize))
                }
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("value stored at key is not a list"),
                )),
            }
        } else {
            // Key does not exist, interpreted as empty list
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
pub mod test_llen {

    use super::Llen;
    use crate::commands::database_mock::{DatabaseMock, TypeSaved};
    use std::collections::LinkedList;

    #[test]
    fn test01_llen_an_existing_list_of_one_element() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key"];
        let encode = Llen::run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":1\r\n".to_string());
    }

    #[test]
    fn test02_llen_an_existing_list_of_many_elements() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("this".to_string());
        new_list.push_back("is".to_string());
        new_list.push_back("a".to_string());
        new_list.push_back("list".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key"];
        let encode = Llen::run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":4\r\n".to_string());
    }

    #[test]
    fn test03_llen_to_key_storing_non_list() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey 10
        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec!["key"];
        let error = Llen::run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value stored at key is not a list".to_string()
        );
    }
}
