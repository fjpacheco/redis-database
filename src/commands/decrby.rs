use std::collections::HashMap;
use crate::native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType};

pub struct Database {
    strings: HashMap<String, String>,
}

impl Database {
    pub fn new() -> Database {
        Database {
            strings: HashMap::new(),
        }
    }

    pub fn get_mut_strings(&mut self) -> Option<&mut HashMap<String, String>> {
        Some(&mut self.strings)
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}


pub struct Decrby;

/// Decrements the number stored at key by decrement. If the key does not exist, it is set
/// to 0 before performing the operation. An error is returned if the key contains a value
/// of the wrong type or contains a string that can not be represented as integer.
///
/// Operation is limited to 64 bit signed integers.

impl Decrby {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        if let Some(strings) = database.get_mut_strings() {
            // get strings hashmap
            let mut decr = String::from(buffer_vec.pop().unwrap()); // extract key and decrement from: Vec<&str> = ["mykey", "10"]
            let key = String::from(buffer_vec.pop().unwrap());

            let decr_int = get_as_integer(&mut decr)?; // check if decr is parsable as int
            let key_cpy = key.clone();
            let current_key_value: isize = string_key_check(strings, key)?;
            let new_value = current_key_value - decr_int; // old_int - decr_int
            strings.insert(key_cpy, new_value.to_string());
            Ok(RInteger::encode(new_value)) // as isize
        } else {
            // strings hashmap get didn't work
            Err(ErrorStruct::new(
                "ERR".to_string(),
                "Weird stuff going on with the Database".to_string(),
            ))
        }
    }
}

pub fn string_key_check(
    strings: &mut HashMap<String, String>,
    key: String,
) -> Result<isize, ErrorStruct> {
    match strings.get_mut(&key) {
        Some(string_value) => {
            // key exists
            get_as_integer(string_value) // check if string hash value is decrementable
        }
        None => {
            // key does not exist
            let key_cpy = key.clone();
            strings.insert(key, "0".to_string());
            get_as_integer(strings.get_mut(&key_cpy).unwrap())
        }
    }
}

pub fn get_as_integer(value: &mut String) -> Result<isize, ErrorStruct> {
    match value.parse::<isize>() {
        Ok(value_int) => Ok(value_int), // if value is parsable as integer
        Err(_) => Err(ErrorStruct::new(
            "ERR".to_string(),
            "value is not an integer or out of range".to_string(),
        )),
    }
}

#[cfg(test)]
pub mod test_decrby {

    use super::*;

    #[test]
    fn test01_decrby_existing_key() {
        let mut data = Database::new();
        {
            // redis> SET mykey "10" ---> "OK"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> DECRBY mykey 3 ---> (integer) 7
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":7\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"7".to_string())
        );
    }

    #[test]
    fn test02_decrby_existing_key_by_negative_integer() {
        let mut data = Database::new();
        {
            // redis> SET mykey "10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> DECRBY mykey -3
        let buffer: Vec<&str> = vec!["mykey", "-3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"13".to_string())
        );
    }

    #[test]
    fn test03_decrby_existing_key_with_negative_integer_value() {
        let mut data = Database::new();
        {
            // redis> SET mykey "-10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "-10".to_string());
        }
        // redis> DECRBY mykey 3
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-13\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"-13".to_string())
        );
    }

    #[test]
    fn test04_decrby_existing_key_with_negative_integer_value_by_negative_integer() {
        let mut data = Database::new();
        {
            // redis> SET mykey "-10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "-10".to_string());
        }
        // redis> DECRBY mykey -3
        let buffer: Vec<&str> = vec!["mykey", "-3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-7\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"-7".to_string())
        );
    }

    #[test]
    fn test05_decrby_non_existing_key() {
        let mut data = Database::new();
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-3\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"-3".to_string())
        );
    }

    #[test]
    fn test06_decrby_existing_key_with_non_decrementable_value() {
        let mut data = Database::new();
        {
            // redis> SET mykey value
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "value".to_string());
        }
        // redis> DECRBY mykey 1
        let buffer: Vec<&str> = vec!["mykey", "value"];
        let error = Decrby::run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }

    #[test]
    fn test07_decrby_existing_key_by_non_integer() {
        let mut data = Database::new();
        {
            // redis> SET mykey 10
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> DECRBY mykey a
        let buffer: Vec<&str> = vec!["mykey", "a"];
        let error = Decrby::run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }
}
