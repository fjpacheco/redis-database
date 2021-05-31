use std::collections::{HashMap, HashSet, LinkedList};

use crate::native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType};

pub struct DatabaseMock {
    elements: HashMap<String, TypeSaved>,
}

#[derive(Debug, PartialEq)]
pub enum TypeSaved {
    String(String),
    List(LinkedList<String>),
    Set(HashSet<String>),
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
    let decr = String::from(buffer_vec.pop().unwrap()); // extract key and decrement from: Vec<&str> = ["mykey", "10"]
    let key = String::from(buffer_vec.pop().unwrap());

    let decr_int = get_as_integer(&decr)?; // check if decr is parsable as int

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
        database.insert(key_cpy, TypeSaved::String("0".to_string()));
        get_as_integer(&"0".to_string())
    }
}

pub fn get_as_integer(value: &str) -> Result<isize, ErrorStruct> {
    // value es mut porque TypeSaved::String() devuelve &mut String
    match value.parse::<isize>() {
        Ok(value_int) => Ok(value_int), // if value is parsable as pointer size integer
        Err(_) => Err(ErrorStruct::new(
            "ERR".to_string(),
            "value is not an integer or out of range".to_string(),
        )),
    }
}

// Lpush and rpush aux

pub fn push_at(
    mut buffer: Vec<&str>,
    database: &mut DatabaseMock,
    fill_list: fn(buffer: Vec<&str>, list: &mut LinkedList<String>),
) -> Result<String, ErrorStruct> {
    let key = String::from(buffer.remove(0));
    let size;
    if let Some(typesaved) = database.get_mut(&key) {
        match typesaved {
            TypeSaved::List(list_of_values) => {
                fill_list(buffer, list_of_values);
                size = list_of_values.len();
                Ok(RInteger::encode(size as isize))
            }
            _ => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("key provided is not from strings"),
            )),
        }
    } else {
        let mut new_list: LinkedList<String> = LinkedList::new();
        fill_list(buffer, &mut new_list);
        size = new_list.len();
        database.insert(key, TypeSaved::List(new_list));
        Ok(RInteger::encode(size as isize))
    }
}

pub fn fill_list_from_top(mut buffer: Vec<&str>, list: &mut LinkedList<String>) {
    while !buffer.is_empty() {
        list.push_front(buffer.remove(0).to_string());
    }
}

pub fn fill_list_from_bottom(mut buffer: Vec<&str>, list: &mut LinkedList<String>) {
    while !buffer.is_empty() {
        list.push_back(buffer.remove(0).to_string());
    }
}
