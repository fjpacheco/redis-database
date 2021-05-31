use crate::commands::database_mock::{Database, TypeSaved};
use crate::native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType};
pub struct Strlen;

/// Returns the length of the string value stored at key. An error is returned when key holds a non-string value.
///
/// Return value: Integer reply: the length of the string at key, or 0 when key does not exist.

impl Strlen {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        let key = String::from(buffer_vec.pop().unwrap());
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::String(old_value) => Ok(RInteger::encode(old_value.len() as isize)),
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from strings"),
                )),
            }
        } else {
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
pub mod test_strlen {

    use super::*;

    #[test]
    fn test01_strlen_existing_key() {
        let mut data = Database::new();
        // redis> SET mykey somevalue ---> "OK"
        data.insert(
            "mykey".to_string(),
            TypeSaved::String("somevalue".to_string()),
        );
        // redis> STRLEN mykey ---> (integer) 9
        let buffer: Vec<&str> = vec!["mykey"];
        let encoded = Strlen::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":9\r\n".to_string());
    }

    #[test]
    fn test02_srlen_non_existing_key() {
        let mut data = Database::new();
        // redis> STRLEN nonexisting ---> (integer) 0
        let buffer: Vec<&str> = vec!["mykey"];
        let encoded = Strlen::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":0\r\n".to_string());
    }
}
