use crate::native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType};

use std::collections::HashMap;

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

pub fn execute_value_modification(
    database: &mut Database,
    mut buffer_vec: Vec<&str>,
    op: fn(isize, isize) -> isize,
) -> Result<String, ErrorStruct> {
    if let Some(strings) = database.get_mut_strings() {
        // get strings hashmap
        let mut decr = String::from(buffer_vec.pop().unwrap()); // extract key and decrement from: Vec<&str> = ["mykey", "10"]
        let key = String::from(buffer_vec.pop().unwrap());
        let decr_int = get_as_integer(&mut decr)?; // check if decr is parsable as int
        let current_key_value: isize = string_key_check(strings, String::from(&key))?;
        let new_value = op(current_key_value, decr_int);
        strings.insert(key, new_value.to_string());
        Ok(RInteger::encode(new_value)) // as isize
    } else {
        Err(ErrorStruct::new(
            // strings hashmap get didn't work
            "ERR".to_string(),
            "Weird stuff going on with the Database".to_string(),
        ))
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
        Ok(value_int) => Ok(value_int), // if value is parsable as pointer size integer
        Err(_) => Err(ErrorStruct::new(
            "ERR".to_string(),
            "value is not an integer or out of range".to_string(),
        )),
    }
}
