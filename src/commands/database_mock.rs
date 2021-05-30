use std::collections::{HashMap, HashSet};

use crate::native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType};

pub struct DatabaseMock {
    elements: HashMap<String, TypeSaved>,
}

#[derive(Debug, PartialEq)]
pub enum TypeSaved {
    String(String),
    Lists(Vec<String>),
    Sets(HashSet<String>),
}

impl DatabaseMock {
    pub fn new() -> Self {
        DatabaseMock {
            elements: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: TypeSaved) -> Option<TypeSaved> {
        self.elements.insert(key, value)
    }

    pub fn get(&mut self, key: &str) -> Option<&TypeSaved> {
        self.elements.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TypeSaved> {
        self.elements.get_mut(key)
    }
}

impl Default for DatabaseMock {
    fn default() -> Self {
        Self::new()
    }
}

// Aux functions

pub fn execute_value_modification(
    database: &mut DatabaseMock,
    mut buffer_vec: Vec<&str>,
    op: fn(isize, isize) -> isize,
) -> Result<String, ErrorStruct> {
    let mut decr = String::from(buffer_vec.pop().unwrap()); // extract key and decrement from: Vec<&str> = ["mykey", "10"]
    let key = String::from(buffer_vec.pop().unwrap());

    let decr_int = get_as_integer(&mut decr)?; // check if decr is parsable as int

    let current_key_value: isize = string_key_check(database, String::from(&key))?;

    let new_value = op(current_key_value, decr_int);
    database.insert(key, TypeSaved::String(new_value.to_string()));
    Ok(RInteger::encode(new_value)) // as isize
}

pub fn string_key_check(database: &mut DatabaseMock, key: String) -> Result<isize, ErrorStruct> {
    if let Some(typesaved) = database.get_mut(&key) {
        match typesaved {
            TypeSaved::String(old_value) => get_as_integer(&old_value),
            _ => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("key provided is not from strings"),
            )),
        }
    } else {
        // key does not exist
        let key_cpy = key.clone();
        database.insert(key_cpy.to_string(), TypeSaved::String("0".to_string()));
        get_as_integer(&mut "0".to_string())
    }
}

pub fn get_as_integer(value: &String) -> Result<isize, ErrorStruct> {
    // value es mut porque TypeSaved::String() devuelve &mut String
    match value.parse::<isize>() {
        Ok(value_int) => Ok(value_int), // if value is parsable as pointer size integer
        Err(_) => Err(ErrorStruct::new(
            "ERR".to_string(),
            "value is not an integer or out of range".to_string(),
        )),
    }
}
