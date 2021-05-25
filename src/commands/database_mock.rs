use std::collections::{HashMap, HashSet};

use crate::{
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

// For mock...
pub enum UnitTypeSaved {
    Strings(String),
    Lists(String),
    Sets(String),
}

// Mock DatabaseMock
pub struct DatabaseMock {
    strings: HashMap<String, String>,
    lists: HashMap<String, Vec<String>>,
    sets: HashMap<String, HashSet<String>>,
}

impl Default for DatabaseMock {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseMock {
    pub fn new() -> Self {
        DatabaseMock {
            strings: HashMap::new(),
            lists: HashMap::new(),
            sets: HashMap::new(),
        }
    }

    pub fn insert_unit(
        &mut self,
        key: String,
        value: UnitTypeSaved,
    ) -> Result<String, ErrorStruct> {
        match value {
            UnitTypeSaved::Strings(value_item) => {
                // Si tenías creada con esa key una lista/set, la reemplazará por un String!
                let _ = self.sets.remove(&key);
                let _ = self.lists.remove(&key);
                let _ = self.strings.insert(key, value_item);
                Ok(RSimpleString::encode(redis_messages::ok()))
            }
            UnitTypeSaved::Lists(_) => Err(ErrorStruct::new(
                "ERR Rust-eze team".to_string(),
                "command not implemented".to_string(),
            )),
            UnitTypeSaved::Sets(_) => Err(ErrorStruct::new(
                "ERR Rust-eze team".to_string(),
                "command not implemented".to_string(),
            )),
        }
    }

    pub fn get_unit_string(&mut self, key: String) -> Result<String, ErrorStruct> {
        match self.strings.get(&key) {
            Some(item) => Ok(item.to_string()),
            None => {
                if self.lists.contains_key(&key) || self.sets.contains_key(&key) {
                    let message_error = redis_messages::wrongtype_in_get_key();
                    Err(ErrorStruct::new(
                        message_error.get_prefix(),
                        message_error.get_message(),
                    ))
                } else {
                    Ok(redis_messages::nil())
                }
            }
        }
    }
}
