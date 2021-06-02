use crate::{
    database::{Database, TypeSaved},
    native_types::{ErrorStruct, RInteger, RedisType},
};

use super::get_as_integer;

pub mod decrby;
pub mod get;
pub mod incrby;
pub mod mget;
pub mod mset;
pub mod set;
pub mod strlen;

pub fn execute_value_modification(
    database: &mut Database,
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

pub fn string_key_check(database: &mut Database, key: String) -> Result<isize, ErrorStruct> {
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
