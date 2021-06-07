use std::collections::VecDeque;

use crate::{
    database::{Database, TypeSaved},
    native_types::{ErrorStruct, RArray, RBulkString, RInteger, RedisType},
};

pub mod lindex;
pub mod llen;
pub mod lpop;
pub mod lpush;
pub mod lpushx;
pub mod lrange;
pub mod lrem;
pub mod lset;
pub mod rpop;
pub mod rpush;
pub mod rpushx;

// Lpush, rpush, lpushx and rpushx aux

pub fn fill_list_from_top(mut buffer: Vec<&str>, list: &mut VecDeque<String>) {
    while !buffer.is_empty() {
        list.push_front(buffer.remove(0).to_string());
    }
}

pub fn fill_list_from_bottom(mut buffer: Vec<&str>, list: &mut VecDeque<String>) {
    while !buffer.is_empty() {
        list.push_back(buffer.remove(0).to_string());
    }
}

// Lpush and rpush aux

pub fn push_at(
    mut buffer: Vec<&str>,
    database: &mut Database,
    fill_list: fn(buffer: Vec<&str>, list: &mut VecDeque<String>),
) -> Result<String, ErrorStruct> {
    check_not_empty(&buffer)?;
    let key = String::from(buffer.remove(0));
    check_not_empty(&buffer)?;
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
        let mut new_list: VecDeque<String> = VecDeque::new();
        fill_list(buffer, &mut new_list);
        size = new_list.len();
        database.insert(key, TypeSaved::List(new_list));
        Ok(RInteger::encode(size as isize))
    }
}

// Lpushx and rpushx aux

pub fn pushx_at(
    mut buffer: Vec<&str>,
    database: &mut Database,
    fill_list: fn(buffer: Vec<&str>, list: &mut VecDeque<String>),
) -> Result<String, ErrorStruct> {
    check_not_empty(&buffer)?;
    let key = String::from(buffer.remove(0));
    check_not_empty(&buffer)?;
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
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("no list found with entered key"),
        ))
    }
}

pub fn pop_at(
    mut buffer: Vec<&str>,
    database: &mut Database,
    fill_list: fn(list: &mut VecDeque<String>, counter: usize) -> String,
) -> Result<String, ErrorStruct> {
    check_not_empty(&buffer)?;
    let key = String::from(buffer.remove(0));
    let count = parse_count(&mut buffer)?;
    check_empty(&buffer)?;
    if let Some(typesaved) = database.get_mut(&key) {
        match typesaved {
            TypeSaved::List(list_of_values) => {
                if count <= list_of_values.len() {
                    Ok(fill_list(list_of_values, count))
                } else {
                    Err(ErrorStruct::new(
                        String::from("ERR"),
                        String::from("argument is not a number or out of index"),
                    ))
                }
            }
            _ => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("key provided is not from strings"),
            )),
        }
    } else {
        Ok(RBulkString::encode("(nil)".to_string()))
    }
}

fn parse_count(buffer: &mut Vec<&str>) -> Result<usize, ErrorStruct> {
    if let Some(value) = buffer.pop() {
        if let Ok(counter) = value.parse::<usize>() {
            if counter > 0 {
                Ok(counter)
            } else {
                Err(ErrorStruct::new(
                    String::from("ERRUSIZE"),
                    String::from("provided counter is not a natural number"),
                ))
            }
        } else {
            Err(ErrorStruct::new(
                String::from("ERRUSIZE"),
                String::from("provided counter is not a natural number"),
            ))
        }
    } else {
        Ok(1)
    }
}

pub fn remove_values_from_top(list: &mut VecDeque<String>, counter: usize) -> String {
    let mut popped: Vec<String> = Vec::new();
    if counter > 1 {
        for _ in 0..counter {
            popped.push(list.pop_front().unwrap());
        }
        RArray::encode(popped)
    } else {
        RBulkString::encode(list.pop_front().unwrap())
    }
}

pub fn remove_values_from_bottom(list: &mut VecDeque<String>, counter: usize) -> String {
    let mut popped: Vec<String> = Vec::new();
    if counter > 1 {
        for _ in 0..counter {
            popped.push(list.pop_back().unwrap());
        }
        RArray::encode(popped)
    } else {
        RBulkString::encode(list.pop_back().unwrap())
    }
}

fn check_not_empty(buffer: &[&str]) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("wrong number of arguments"),
        ))
    } else {
        Ok(())
    }
}

fn check_empty(buffer: &[&str]) -> Result<(), ErrorStruct> {
    if !buffer.is_empty() {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("wrong number of arguments"),
        ))
    } else {
        Ok(())
    }
}
