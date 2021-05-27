use std::collections::HashMap;

use super::database::{database_check, Database};
use crate::native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType};

pub struct Strlen;

/// Returns the length of the string value stored at key. An error is returned when key holds a non-string value.
///
/// Return value: Integer reply: the length of the string at key, or 0 when key does not exist.

impl Strlen {
    // Error because of non-string value is not contemplated
    pub fn run(buffer_vec: &mut Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        let strings: &mut HashMap<String, String> = database_check(database)?; // get strings hashmap
        let key = String::from(buffer_vec.pop().unwrap()); // extract key from: Vec<&str> = ["mykey"]
        match strings.get_mut(&key) {
            Some(string_value) => {
                // key exists
                Ok(RInteger::encode(string_value.len() as isize)) // check if string hash value is decrementable
            }
            None => {
                // key does not exist
                Ok(RInteger::encode(0))
            }
        }
    }
}

#[cfg(test)]
pub mod test_strlen {

    use super::*;

    #[test]
    fn test01_strlen_existing_key() {
        let mut data = Database::new();
        {
            // redis> SET mykey somevalue ---> "OK"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "somevalue".to_string());
        }
        // redis> STRLEN mykey ---> (integer) 9
        let mut buffer: Vec<&str> = vec!["mykey"];
        let encoded = Strlen::run(&mut buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":9\r\n".to_string());
    }

    #[test]
    fn test02_incrby_non_existing_key() {
        let mut data = Database::new();
        // redis> STRLEN nonexisting ---> (integer) 0
        let mut buffer: Vec<&str> = vec!["mykey"];
        let encoded = Strlen::run(&mut buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":0\r\n".to_string());
    }
}
