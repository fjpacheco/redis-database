use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::regex::super_regex::SuperRegex;
use crate::time_expiration::expire_info::ExpireInfo;
use crate::commands::server::info_formatter::info_db_formatter;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use regex;

extern crate rand;
use rand::seq::IteratorRandom;

pub struct Database {
    elements: HashMap<String, (ExpireInfo, TypeSaved)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeSaved {
    String(String),
    List(VecDeque<String>),
    Set(HashSet<String>),
}

impl Database {
    pub fn new() -> Self {
        Database {
            elements: HashMap::new(),
        }
    }

    pub fn info(&self) -> Result<Vec<String>, ErrorStruct> {
        let mut info = Vec::new();
        info.push(info_db_formatter::title());
        info.push(info_db_formatter::number_of_keys(self.elements.len()));
        Ok(info)
    }

    pub fn remove(&mut self, key: &str) -> Option<TypeSaved> {
        if let Some((_, value)) = self.elements.remove(key) {
            Some(value)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: String, value: TypeSaved) -> Option<TypeSaved> {
        if let Some((_, value)) = self.elements.insert(key, (ExpireInfo::new(), value)) {
            Some(value)
        } else {
            None
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&TypeSaved> {
        self.touch(key);
        if let Some((_, value)) = self.elements.get(key) {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TypeSaved> {
        self.touch(key);
        if let Some((_, value)) = self.elements.get_mut(key) {
            Some(value)
        } else {
            None
        }
    }

    pub fn contains_key(&mut self, key: &str) -> bool {
        self.touch(key);
        self.elements.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn touch(&mut self, key: &str) -> bool {
        if let Some((info, _)) = self.elements.get_mut(key) {
            if info.is_expired() {
                self.elements.remove(key);
                return true;
            }
        }
        false
    }

    pub fn ttl(&mut self, key: &str) -> Option<u64> {
        self.touch(key);
        if let Some((info, _)) = self.elements.get(key) {
            info.ttl()
        } else {
            None
        }
    }

    pub fn set_ttl(&mut self, key: &str, timeout: u64) -> Result<(), ErrorStruct> {
        self.touch(key);
        if let Some((info, _)) = self.elements.get_mut(key) {
            info.set_timeout(timeout)?;
            Ok(())
        } else {
            let message = redis_messages::key_not_found();
            Err(ErrorStruct::new(
                message.get_prefix(),
                message.get_message(),
            ))
        }
    }

    pub fn set_ttl_unix_timestamp(&mut self, key: &str, timeout: u64) -> Result<(), ErrorStruct> {
        self.touch(key);
        if let Some((info, _)) = self.elements.get_mut(key) {
            info.set_timeout_unix_timestamp(timeout)?;
            Ok(())
        } else {
            let message = redis_messages::key_not_found();
            Err(ErrorStruct::new(
                message.get_prefix(),
                message.get_message(),
            ))
        }
    }

    pub fn persist(&mut self, key: &str) -> Option<u64> {
        self.touch(key);
        if let Some((info, _)) = self.elements.get_mut(key) {
            info.persist()
        } else {
            None
        }
    }

    pub fn random_key(&mut self) -> Option<String> {
        let mut rng = rand::thread_rng();
        self.elements.keys().choose(&mut rng).map(String::from)
    }

    pub fn match_pattern(&self, regex: &str) -> Result<Vec<String>, regex::Error> {
        let matcher = SuperRegex::from(regex)?;

        Ok(self
            .elements
            .keys()
            .filter(|key| matcher.is_match(key))
            .map(String::from)
            .collect())
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database")
    }
}

#[cfg(test)]
mod test_database {

    use super::*;

    #[test]
    fn test01_insert_a_key() {
        let mut database = Database::new();
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        let got = database.get("key");
        match got.unwrap() {
            TypeSaved::String(value) => {
                assert_eq!(value, "hola");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test02_remove_a_key() {
        let mut database = Database::new();
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.remove("key");
        let got = database.get("key");
        assert_eq!(got, None);
    }

    #[test]
    fn test03_database_contains_key() {
        let mut database = Database::new();
        assert!(!database.contains_key("key"));
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        assert!(database.contains_key("key"));
    }

    #[test]
    fn test04_set_timeout_for_existing_key() {
        let mut database = Database::new();
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.set_ttl("key", 10).unwrap();
        assert_eq!(database.ttl("key"), Some(9));
    }

    #[test]
    fn test05_set_timeout_for_non_existing_key() {
        let mut database = Database::new();
        match database.set_ttl("key", 10) {
            Err(should_throw_error) => assert_eq!(
                should_throw_error.print_it(),
                "KEYNOTFOUND Session does not exist or has timed out".to_string()
            ),
            Ok(()) => {}
        }
    }

    #[test]
    fn test06_set_timeout_for_key_and_let_it_persist() {
        let mut database = Database::new();
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.set_ttl("key", 10).unwrap();
        assert_eq!(database.persist("key"), Some(9));
        assert_eq!(database.ttl("key"), None);
    }
}
