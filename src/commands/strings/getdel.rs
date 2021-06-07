use crate::commands::Runnable;
use crate::database::{Database, TypeSaved};
use crate::native_types::bulk_string::RBulkString;
use crate::native_types::error::ErrorStruct;
use crate::native_types::redis_type::RedisType;

use super::{no_more_values, pop_value};

pub struct Getdel;

impl Runnable for Getdel {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        let key = pop_value(&mut buffer_vec)?;
        no_more_values(&buffer_vec, "getdel")?;

        if let Some(value) = database.remove(&key) {
            match value {
                TypeSaved::String(value) => Ok(RBulkString::encode(value)),
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from string"),
                )),
            }
        } else {
            Ok(RBulkString::encode("(nil)".to_string()))
        }
    }
}

#[cfg(test)]
pub mod test_getdel {

    use super::*;
    use crate::database::{Database, TypeSaved};

    #[test]
    fn test01_getdel_of_an_existing_key() {
        let mut data = Database::new();

        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer: Vec<&str> = vec!["key"];
        let encoded = Getdel.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), "$5\r\nvalue\r\n".to_string());
        assert_eq!(data.get("key"), None);
    }

    #[test]
    fn test02_getdel_of_a_non_existing_key() {
        let mut data = Database::new();
        let buffer: Vec<&str> = vec!["key"];
        let encoded = Getdel.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), "$-1\r\n".to_string());
        assert_eq!(data.get("key"), None);
    }

    #[test]
    fn test03_wrong_number_of_arguments() {
        let mut data = Database::new();

        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer: Vec<&str> = vec!["key", "ahre", "mas", "argumentos"];
        let encoded = Getdel.run(buffer, &mut data);
        match encoded {
            Ok(_value) => {}
            Err(error) => assert_eq!(
                error.print_it(),
                "ERR wrong number of arguments for 'getdel' command"
            ),
        }
    }
}
