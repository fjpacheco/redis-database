use crate::commands::Runnable;
use crate::native_types::bulk_string::RBulkString;
use crate::native_types::error::ErrorStruct;
use crate::native_types::redis_type::RedisType;

use super::{no_more_values, pop_value, replace_value};
use crate::database::{Database, TypeSaved};

pub struct Getset;

impl Runnable for Getset {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        let new_value = pop_value(&mut buffer_vec)?;
        let key = pop_value(&mut buffer_vec)?;
        no_more_values(&buffer_vec, "getset")?;

        if let Some(typesaved) = database.get(&key) {
            match typesaved {
                TypeSaved::String(_old_value) => replace_value(database, key, new_value),
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided does not match an string"),
                )),
            }
        } else {
            database.insert(key, TypeSaved::String(new_value));
            Ok(RBulkString::encode("(nil)".to_string()))
        }
    }
}

#[cfg(test)]
pub mod test_getset {

    use super::*;
    use crate::database::{Database, TypeSaved};
    #[test]
    fn test01_getset_of_an_existing_key() {
        let mut data = Database::new();
        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer: Vec<&str> = vec!["key", "other"];
        let encoded = Getset.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), "$5\r\nvalue\r\n".to_string());
        assert_eq!(
            data.get("key"),
            Some(&TypeSaved::String("other".to_string()))
        );
    }

    #[test]
    fn test02_getset_of_a_non_existing_key() {
        let mut data = Database::new();
        let buffer: Vec<&str> = vec!["key", "newValue"];
        let encoded = Getset.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), "$-1\r\n".to_string());
        assert_eq!(
            data.get("key"),
            Some(&TypeSaved::String("newValue".to_string()))
        );
    }

    #[test]
    fn test03_wrong_number_of_arguments() {
        let mut data = Database::new();

        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer: Vec<&str> = vec![];
        let encoded = Getset.run(buffer, &mut data);
        match encoded {
            Ok(_value) => {}
            Err(error) => assert_eq!(
                error.print_it(),
                "ERR wrong number of arguments for 'append' command"
            ),
        }
    }
}
