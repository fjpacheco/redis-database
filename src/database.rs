use crate::commands::server::info_formatter::info_db_formatter;
use crate::native_types::error::ErrorStruct;
use crate::regex::super_regex::SuperRegex;
use crate::time_expiration::expire_info::ExpireInfo;
use crate::{messages::redis_messages, tcp_protocol::notifier::Notifier};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex};

use regex;

extern crate rand;
use rand::seq::IteratorRandom;

pub struct Database {
    elements: HashMap<String, (ExpireInfo, TypeSaved)>,
    notifier: Arc<Mutex<Notifier>>, // https://stackoverflow.com/questions/40384274/rust-mpscsender-cannot-be-shared-between-threads
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeSaved {
    String(String),
    List(VecDeque<String>),
    Set(HashSet<String>),
}

impl Database {
    pub fn new(notifier: Notifier) -> Self {
        Database {
            elements: HashMap::new(),
            notifier: Arc::new(Mutex::new(notifier)),
        }
    }

    pub fn info(&self) -> Result<Vec<String>, ErrorStruct> {
        Ok(vec![
            info_db_formatter::title(),
            info_db_formatter::number_of_keys(self.elements.len()),
        ])
    }

    pub fn size(&self) -> usize {
        self.elements.len()
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
        let _ = self.private_touch(key, None);
        if let Some((_, value)) = self.elements.get(key) {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TypeSaved> {
        let _ = self.private_touch(key, None);
        if let Some((_, value)) = self.elements.get_mut(key) {
            Some(value)
        } else {
            None
        }
    }

    pub fn contains_key(&mut self, key: &str) -> bool {
        let _ = self.private_touch(key, None);
        self.elements.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    fn private_touch(
        &mut self,
        key: &str,
        notifier: Option<Arc<Mutex<Notifier>>>,
    ) -> Result<bool, ErrorStruct> {
        if let Some((info, _)) = self.elements.get_mut(key) {
            if info.is_expired(notifier, key) {
                self.elements.remove(key);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(ErrorStruct::from(redis_messages::key_not_found()))
        }
    }

    pub fn touch(&mut self, key: &str) -> Result<bool, ErrorStruct> {
        self.private_touch(
            key,
            Some(Arc::clone(&self.notifier)), /*HERE GOES THE NOTIFIER*/
        )
    }

    pub fn ttl(&mut self, key: &str) -> Option<u64> {
        let _ = self.private_touch(key, None);
        if let Some((info, _)) = self.elements.get(key) {
            info.ttl()
        } else {
            None
        }
    }

    pub fn set_ttl(&mut self, key: &str, timeout: u64) -> Result<(), ErrorStruct> {
        let _ = self.private_touch(key, None);
        if let Some((info, _)) = self.elements.get_mut(key) {
            info.set_timeout(timeout)?;
            println!("asd");
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
        let _ = self.private_touch(key, None);
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
        let _ = self.private_touch(key, None);
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

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database")
    }
}

#[cfg(test)]
mod test_database {

    use crate::commands::create_notifier;

    use super::*;

    #[test]
    fn test01_insert_a_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
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
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.remove("key");
        let got = database.get("key");
        assert_eq!(got, None);
    }

    #[test]
    fn test03_database_contains_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        assert!(!database.contains_key("key"));
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        assert!(database.contains_key("key"));
    }

    #[test]
    fn test04_set_timeout_for_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.set_ttl("key", 10).unwrap();
        assert_eq!(database.ttl("key"), Some(9));
    }

    #[test]
    fn test05_set_timeout_for_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
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
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.set_ttl("key", 10).unwrap();
        assert_eq!(database.persist("key"), Some(9));
        assert_eq!(database.ttl("key"), None);
    }
}
