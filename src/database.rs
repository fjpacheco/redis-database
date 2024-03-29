use crate::commands::server::info_formatter::info_db_formatter;
use crate::native_types::error::ErrorStruct;
use crate::native_types::{RArray, RInteger, RSimpleString, RedisType};
use crate::redis_config;
use crate::regex::super_regex::SuperRegex;
use crate::time_expiration::expire_info::ExpireInfo;
use crate::{messages::redis_messages, tcp_protocol::notifier::Notifier};
use std::fmt;
use std::io::Lines;
use std::sync::{Arc, Mutex};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::Not,
};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

extern crate rand;
use rand::seq::IteratorRandom;
use redis_config::RedisConfig;

pub struct Database {
    elements: HashMap<String, (ExpireInfo, TypeSaved)>,
    redis_config: Option<Arc<Mutex<RedisConfig>>>,
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
            redis_config: None,
        }
    }

    /// Database Redis Config setter
    pub fn set_redis_config(&mut self, redis_config: Arc<Mutex<RedisConfig>>) {
        self.redis_config = Some(redis_config);
    }

    /// Creates a new instance of the Database given a specified RedisConfig
    /// This method playes an important role for restoring the Database.
    pub fn new_from(
        config: Arc<Mutex<RedisConfig>>,
        notifier: Notifier,
    ) -> Result<Self, ErrorStruct> {
        let mut elements = HashMap::new();
        let file = File::open(
            config
                .lock()
                .map_err(|_| {
                    ErrorStruct::from(redis_messages::poisoned_lock(
                        "redis config",
                        crate::native_types::error_severity::ErrorSeverity::ShutdownServer,
                    ))
                })?
                .db_filename(),
        )
        .map_err(|_| {
            ErrorStruct::from(redis_messages::init_failed(
                "dbfile name",
                crate::native_types::error_severity::ErrorSeverity::ShutdownServer,
            ))
        })?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next() {
            match line {
                Ok(line) => {
                    let expire_info = get_expire_info(line.clone(), &mut lines)?;
                    let type_decoded = decode_case(&mut lines)?;
                    let key_decoded = decode_key(&mut lines)?;
                    let value_decoded = decode_value(&mut lines, type_decoded)?;
                    elements.insert(key_decoded, (expire_info, value_decoded));
                }
                Err(_) => return Err(ErrorStruct::from(redis_messages::file_read_error())),
            }
        }

        Ok(Database {
            elements,
            redis_config: Some(config),
            notifier: Arc::new(Mutex::new(notifier)),
        })
    }

    /// Returns a vector with a title for the database and its number
    /// of keys.
    pub fn info(&self) -> Result<Vec<String>, ErrorStruct> {
        Ok(vec![
            info_db_formatter::title(),
            info_db_formatter::number_of_keys(self.elements.len()),
        ])
    }

    /// Database size getter
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// Removes a specified key from the database.
    pub fn remove(&mut self, key: &str) -> Option<TypeSaved> {
        if let Some((_, value)) = self.elements.remove(key) {
            Some(value)
        } else {
            None
        }
    }

    /// Inserts a key-value pair to the database.
    pub fn insert(&mut self, key: String, value: TypeSaved) -> Option<TypeSaved> {
        if let Some((_, value)) = self.elements.insert(key, (ExpireInfo::new(), value)) {
            Some(value)
        } else {
            None
        }
    }

    /// Database value getter. Important: performs a touch.
    pub fn get(&mut self, key: &str) -> Option<&TypeSaved> {
        let _ = self.private_touch(key, None);
        if let Some((_, value)) = self.elements.get(key) {
            Some(value)
        } else {
            None
        }
    }

    /// Database value mutable getter. Important: performs a touch.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut TypeSaved> {
        let _ = self.private_touch(key, None);
        if let Some((_, value)) = self.elements.get_mut(key) {
            Some(value)
        } else {
            None
        }
    }

    /// Returns true if the database contains the received key.
    /// Important: performs a touch.
    pub fn contains_key(&mut self, key: &str) -> bool {
        let _ = self.private_touch(key, None);
        self.elements.contains_key(key)
    }

    /// Empties the database HashMap.
    pub fn clear(&mut self) {
        self.elements.clear();
    }

    /// Checks if a key has already expired, in that case, it removes it and returns true.
    /// If the key exists but has not expired yet, returns false. If the key does not exist,
    /// throws an error.
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

    /// Performs a touch calling private_touch().
    pub fn touch(&mut self, key: &str) -> Result<bool, ErrorStruct> {
        self.private_touch(
            key,
            Some(Arc::clone(&self.notifier)), /*HERE GOES THE NOTIFIER*/
        )
    }

    /// Returns the timeout of a specified key. Important: performs a touch.
    pub fn ttl(&mut self, key: &str) -> Option<u64> {
        let _ = self.private_touch(key, None);
        if let Some((info, _)) = self.elements.get(key) {
            info.ttl()
        } else {
            None
        }
    }

    /// Database keys timeout setter. Important: performs a touch.
    pub fn set_ttl(&mut self, key: &str, timeout: u64) -> Result<(), ErrorStruct> {
        let _ = self.private_touch(key, None);
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

    /// Database keys unix timestamp timeout setter. Important: performs a touch.
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

    /// Given a key, tries to obtain its tuple value and check for its ExpireInfo
    /// if it actually IS expirable, calls persist() and returns its timeout. See
    /// ExpireInfo persist() for a deeper understanding. Important: performs a touch.
    pub fn persist(&mut self, key: &str) -> Option<u64> {
        let _ = self.private_touch(key, None);
        if let Some((info, _)) = self.elements.get_mut(key) {
            info.persist()
        } else {
            None
        }
    }

    /// Returns a random key from the database using Rust Rand module method choose().
    pub fn random_key(&mut self) -> Option<String> {
        let mut rng = rand::thread_rng();
        self.elements.keys().choose(&mut rng).map(String::from)
    }

    /// Writes to a file current information about all keys and values ​​in the database
    /// including their expiration time and type. See SAVE command.
    /// This method is useful for restoring the database.
    ///
    /// File format: :{EXPIRE_TIME}:{CASE}+{KEY}+{VALUE}
    /// Where:
    /// * EXPIRE_TIME can be any positive value or -1 if its not an expirable key
    /// encoded as Redis Integer.
    /// * CASE: 0: String, 1: List, 2: Set encoded as Redis Integer.
    /// * KEY: Redis Simple String.
    /// * VALUE: Redis Simple String or Redis Array.
    pub fn take_snapshot(&mut self) -> Result<(), ErrorStruct> {
        let mut config = if let Some(config) = self.redis_config.as_ref() {
            config.lock().map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "redis config",
                    crate::native_types::error_severity::ErrorSeverity::ShutdownServer,
                ))
            })?
        } else {
            return Err(ErrorStruct::from(redis_messages::unexpected_behaviour(
                "no redis config available",
            )));
        };
        let mut file = config.get_mut_dump_file().unwrap();
        for (key, (expire_info, typesaved)) in self.elements.iter_mut() {
            let mut expire_clone = expire_info.clone();
            if expire_clone
                .is_expired(Some(self.notifier.clone()), key)
                .not()
            {
                let time = expire_clone.ttl().map(|t| t as isize).unwrap_or(-1);
                write_integer_to_file(time, &mut file)?;
                persist_data(key, &mut file, typesaved)?;
            }
        }
        Ok(())
    }

    /// Returns all database keys matching the pattern received.
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

// Given the lines received moves to the next one, checks if the line is valid
// and returns a TypeSaved obtained from decoding a value read which can be
// a Redis Simple String or a Redis Array.
fn decode_value(
    lines: &mut Lines<BufReader<File>>,
    type_decoded: isize,
) -> Result<TypeSaved, ErrorStruct> {
    if let Some(line) = lines.next() {
        match line {
            Ok(line) => get_matching_typesaved(type_decoded, line, lines),
            Err(_) => Err(ErrorStruct::from(redis_messages::file_read_error())),
        }
    } else {
        Err(ErrorStruct::from(redis_messages::file_read_error()))
    }
}

/// Obtains a specific typesaved according to the type_decoded isize received.
fn get_matching_typesaved(
    type_decoded: isize,
    mut line: String,
    lines: &mut Lines<BufReader<File>>,
) -> Result<TypeSaved, ErrorStruct> {
    match type_decoded {
        0 => {
            check_decodable_line(&mut line, '+')?;
            let value = match RSimpleString::decode(line, lines) {
                Ok(value) => value,
                Err(err) => return Err(err),
            };
            Ok(TypeSaved::String(value))
        }
        1 => {
            check_decodable_line(&mut line, '*')?;
            let value = match RArray::decode(line, lines) {
                Ok(value) => value,
                Err(err) => return Err(err),
            };
            Ok(TypeSaved::List(VecDeque::from(value)))
        }
        _ => {
            check_decodable_line(&mut line, '*')?;
            let value = match RArray::decode(line, lines) {
                Ok(value) => value,
                Err(err) => return Err(err),
            };
            Ok(TypeSaved::Set(value.into_iter().collect()))
        }
    }
}

/// Given the lines received moves to the next one, checks if the line is valid
/// and returns a key String.
fn decode_key(lines: &mut Lines<BufReader<File>>) -> Result<String, ErrorStruct> {
    if let Some(line) = lines.next() {
        match line {
            Ok(mut line) => {
                check_decodable_line(&mut line, '+')?;
                RSimpleString::decode(line, lines)
            }
            Err(_) => Err(ErrorStruct::from(redis_messages::file_read_error())),
        }
    } else {
        Err(ErrorStruct::from(redis_messages::file_read_error()))
    }
}

/// Given the lines received moves to the next one, checks if the line is valid
/// and returns an isize (0, 1 or 2) identifying the case (String, List or Set).
fn decode_case(lines: &mut Lines<BufReader<File>>) -> Result<isize, ErrorStruct> {
    if let Some(line) = lines.next() {
        match line {
            Ok(mut line) => {
                check_decodable_line(&mut line, ':')?;
                get_case(line, lines)
            }
            Err(_) => Err(ErrorStruct::from(redis_messages::file_read_error())),
        }
    } else {
        Err(ErrorStruct::from(redis_messages::file_read_error()))
    }
}

/// Obtains an isize from the parameters received and returns it if it matches any
/// of the 3 possible cases (0: String, 1: List, 2: Set). Any other case, returns error.
fn get_case(line: String, lines: &mut Lines<BufReader<File>>) -> Result<isize, ErrorStruct> {
    let value = RInteger::decode(line, lines)?;
    if value == 0 || value == 1 || value == 2 {
        return Ok(value);
    }
    Err(ErrorStruct::from(redis_messages::unexpected_behaviour(
        "unknown case found at dump",
    )))
}

/// Given a string line and its following ones obtains a timeout and returns an instance
/// of ExpireInfo.
fn get_expire_info(
    mut line: String,
    lines: &mut Lines<BufReader<File>>,
) -> Result<ExpireInfo, ErrorStruct> {
    check_decodable_line(&mut line, ':')?;
    let ttl_decoded = RInteger::decode(line, lines)?;
    let mut expire_info: ExpireInfo = ExpireInfo::new();
    if ttl_decoded >= 0 {
        expire_info.set_timeout(ttl_decoded as u64)?;
    }
    Ok(expire_info)
}

/// Checks if the first character of the given string line matches the received character.
fn check_decodable_line(line: &mut String, char: char) -> Result<(), ErrorStruct> {
    if line.remove(0) != char {
        return Err(ErrorStruct::from(redis_messages::unexpected_behaviour(
            "unexpected char found at dump",
        )));
    }
    Ok(())
}

/// Performs the writing of a String to the given file, while first encoding it as
/// a Redis Simple String (RSimpleString). Returns error in case writing failed.
fn write_string_to_file(string: &str, file: &mut File) -> Result<(), ErrorStruct> {
    file.write_all(RSimpleString::encode(string.to_string()).as_bytes())
        .map_err(|_| ErrorStruct::from(redis_messages::write_error()))
}

/// Performs the writing of an isize to the given file, while first encoding it
/// as a Redis Integer (RInteger). Returns error in case writing failed.
fn write_integer_to_file(number: isize, file: &mut File) -> Result<(), ErrorStruct> {
    file.write_all(RInteger::encode(number).as_bytes())
        .map_err(|_| ErrorStruct::from(redis_messages::write_error()))
}

/// Performs the writing of a Vec<String> to the given file, while first encoding it
/// as a Redis Array (RArray). Returns error in case writing failed.
fn write_array_to_file(vector: Vec<String>, file: &mut File) -> Result<(), ErrorStruct> {
    file.write_all(RArray::encode(vector).as_bytes())
        .map_err(|_| ErrorStruct::from(redis_messages::write_error()))
}

enum TypeCase {
    String = 0,
    List = 1,
    Set = 2,
}

/// Auxiliar function which performs the writing of a specified pair key-value of the database
/// to the received file using the established file format: ":{EXPIRE_TIME}:{CASE}+{KEY}+{VALUE}"
/// Where:
/// * EXPIRE_TIME can be any positive value or -1 if its not an expirable key
/// encoded as Redis Integer.
/// * CASE: 0: String, 1: List, 2: Set encoded as Redis Integer.
/// * KEY: Redis Simple String.
/// * VALUE: Redis Simple String or Redis Array.
fn persist_data(key: &str, file: &mut File, typesaved: &TypeSaved) -> Result<(), ErrorStruct> {
    match typesaved {
        TypeSaved::String(value) => {
            write_integer_to_file(TypeCase::String as isize, file)?; // 0: String Encoding
            write_string_to_file(key, file)?; // KEY encoded as Redis String
            write_string_to_file(value, file)?;
        }
        TypeSaved::List(values) => {
            write_integer_to_file(TypeCase::List as isize, file)?; // 1: List Encoding
            write_string_to_file(key, file)?; // KEY encoded as Redis String
            let vector: Vec<String> = values.iter().map(|member| member.to_string()).collect();
            write_array_to_file(vector, file)?;
        }
        TypeSaved::Set(values) => {
            write_integer_to_file(TypeCase::Set as isize, file)?; // 2: Set Encoding
            write_string_to_file(key, file)?; // KEY encoded as Redis String
            let vector: Vec<String> = values.iter().map(|member| member.to_string()).collect();
            write_array_to_file(vector, file)?;
        }
    };
    Ok(())
}

#[cfg(test)]
mod test_database {

    use super::*;
    use crate::{
        commands::{
            create_notifier,
            lists::{llen::Llen, lpop::LPop, rpop::RPop, rpush::RPush},
            sets::{sadd::Sadd, sismember::Sismember},
            strings::{get::Get, set::Set},
            Runnable,
        },
        native_types::RBulkString,
        vec_strings,
    };
    use std::fs;

    #[test]
    fn test_01_insert_a_key() {
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
    fn test_02_remove_a_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.remove("key");
        let got = database.get("key");
        assert_eq!(got, None);
    }

    #[test]
    fn test_03_database_contains_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        assert!(!database.contains_key("key"));
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        assert!(database.contains_key("key"));
    }

    #[test]
    fn test_04_set_timeout_for_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.set_ttl("key", 10).unwrap();
        assert_eq!(database.ttl("key"), Some(9));
    }

    #[test]
    fn test_05_set_timeout_for_non_existing_key() {
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
    fn test_06_set_timeout_for_key_and_let_it_persist() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        let value = TypeSaved::String(String::from("hola"));
        database.insert("key".to_string(), value);
        database.set_ttl("key", 10).unwrap();
        assert_eq!(database.persist("key"), Some(9));
        assert_eq!(database.ttl("key"), None);
    }

    #[test]
    fn test_07_persist_string_values_at_file() {
        let filename = "database_01.rdb";
        let config = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut original_database = Database::new(notifier);
        original_database.set_redis_config(config);
        original_database.insert(
            "key1".to_string(),
            TypeSaved::String(String::from("value1")),
        );

        original_database.take_snapshot().unwrap();

        assert_eq!(
            fs::read("database_01.rdb").unwrap(),
            b":-1\r\n:0\r\n+key1\r\n+value1\r\n"
        );
    }

    #[test]
    fn test_08_persist_expirable_string_values_at_file() {
        let filename = "database_08.rdb";
        let config = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        database.set_redis_config(config);
        database.insert("key".to_string(), TypeSaved::String(String::from("value")));
        database.set_ttl("key", 5).unwrap();

        database.take_snapshot().unwrap();

        assert_eq!(
            fs::read("database_08.rdb").unwrap(),
            b":4\r\n:0\r\n+key\r\n+value\r\n"
        );
    }

    #[test]
    fn test_09_persist_list_values_at_file() {
        let filename = "database_09.rdb";
        let config = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database.lock().unwrap().set_redis_config(config);
        let buffer = vec_strings!["key", "value1", "value2", "value3", "value4"];
        RPush.run(buffer, &mut database).unwrap();
        database.lock().unwrap().take_snapshot().unwrap();

        assert_eq!(
            fs::read("database_09.rdb").unwrap(),
            b":-1\r\n:1\r\n+key\r\n*4\r\n$6\r\nvalue1\r\n$6\r\nvalue2\r\n$6\r\nvalue3\r\n$6\r\nvalue4\r\n"
        );
    }

    #[ignore]
    #[test]
    fn test_10_persist_set_values_at_file() {
        let filename = "database_10.rdb";
        let config = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier)));
        database.lock().unwrap().set_redis_config(config);
        let buffer = vec_strings!["key", "value1", "value2"];
        Sadd.run(buffer, &mut database).unwrap();
        database.lock().unwrap().take_snapshot().unwrap();

        assert_eq!(
            fs::read("database_10.rdb").unwrap(),
            b":-1\r\n:2\r\n+key\r\n*2\r\n$6\r\nvalue1\r\n$6\r\nvalue2\r\n"
        );
    }

    #[test]
    fn test_11_restore_string_values_from_file() {
        let filename1 = "database_11_a.rdb";
        let config1 = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename1),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut original_database = Database::new(notifier.clone());
        original_database.set_redis_config(config1.clone());
        original_database.insert(
            "key1".to_string(),
            TypeSaved::String(String::from("value1")),
        );
        original_database.take_snapshot().unwrap();

        let filename2 = "database_11_b.rdb";
        let config2 = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename2),
                0,
            )
            .unwrap(),
        ));

        let mut restored_database = Database::new_from(config1, notifier).unwrap();
        restored_database.set_redis_config(config2);
        restored_database.take_snapshot().unwrap();

        assert_eq!(
            fs::read("database_11_a.rdb").unwrap(),
            fs::read("database_11_b.rdb").unwrap()
        );
    }

    #[test]
    fn test_12_restore_list_values_from_file() {
        let filename1 = "database_12_a.rdb";
        let config1 = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename1),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier.clone())));
        database.lock().unwrap().set_redis_config(config1.clone());
        let buffer = vec_strings!["key", "value1", "value2", "value3", "value4"];
        RPush.run(buffer, &mut database).unwrap();

        database.lock().unwrap().take_snapshot().unwrap();

        let filename2 = "database_12_b.rdb";
        let mut restored_database = Database::new_from(config1, notifier).unwrap();

        let config2 = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename2),
                0,
            )
            .unwrap(),
        ));
        restored_database.set_redis_config(config2);

        restored_database.take_snapshot().unwrap();

        assert_eq!(
            fs::read("database_12_a.rdb").unwrap(),
            fs::read("database_12_b.rdb").unwrap()
        );
    }

    #[test]
    fn test_13_restore_set_values_from_file() {
        let filename1 = "database_13_a.rdb";
        let config1 = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename1),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Arc::new(Mutex::new(Database::new(notifier.clone())));
        database.lock().unwrap().set_redis_config(config1.clone());

        let buffer: Vec<String> = vec_strings!["key", "value1", "value2"];
        Sadd.run(buffer, &mut database).unwrap();

        database.lock().unwrap().take_snapshot().unwrap();

        let filename2 = "database_13_b.rdb";
        let config2 = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename2),
                0,
            )
            .unwrap(),
        ));

        let mut restored_database =
            Arc::new(Mutex::new(Database::new_from(config1, notifier).unwrap()));
        database.lock().unwrap().set_redis_config(config2);
        database.lock().unwrap().take_snapshot().unwrap();

        let buffer_mock = vec_strings!["key", "value1"];
        let result_received = Sismember.run(buffer_mock, &mut restored_database);
        let excepted = RInteger::encode(1);
        assert_eq!(excepted, result_received.unwrap());

        assert_eq!(
            Sismember
                .run(vec_strings!["key", "value2"], &mut restored_database)
                .unwrap(),
            RInteger::encode(1)
        );
    }

    #[ignore]
    #[test]
    fn test_14_restore_expirable_string_values_from_file() {
        let filename1 = "database_14_a.rdb";
        let config1 = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename1),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut original_database = Database::new(notifier.clone());
        original_database.set_redis_config(config1);

        original_database.insert("key".to_string(), TypeSaved::String(String::from("value")));
        original_database.set_ttl("key", 2).unwrap();

        original_database.take_snapshot().unwrap();

        let filename2 = "database_14_b.rdb";
        let config2 = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename2),
                0,
            )
            .unwrap(),
        ));
        let mut restored_database = Database::new_from(config2.clone(), notifier).unwrap();
        restored_database.set_redis_config(config2);
        restored_database.take_snapshot().unwrap();

        assert_eq!(
            fs::read("database_14_a.rdb").unwrap(),
            fs::read("database_14_b.rdb").unwrap()
        );
    }
    #[test]
    fn test_15_persist_different_values_at_file() {
        let filename = "database_15.rdb";
        let config = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("log.txt"),
                String::from(filename),
                0,
            )
            .unwrap(),
        ));
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut original_database = Arc::new(Mutex::new(Database::new(notifier.clone())));
        original_database
            .lock()
            .unwrap()
            .set_redis_config(config.clone());

        let buffer1 = vec_strings!["key1", "value1"];
        Set.run(buffer1, &mut original_database).unwrap();

        let buffer2 = vec_strings!["key2", "value1", "value2", "value3", "value4"];
        RPush.run(buffer2, &mut original_database).unwrap();

        let buffer3 = vec_strings!["key3", "value1"];
        Sadd.run(buffer3, &mut original_database).unwrap();

        let _ = original_database.lock().unwrap().take_snapshot();

        let mut restored_database = Arc::new(Mutex::new(
            Database::new_from(config.clone(), notifier).unwrap(),
        ));

        // String value check
        let obtained_value = Get
            .run(vec_strings!["key1"], &mut restored_database)
            .unwrap();
        assert_eq!(obtained_value, RBulkString::encode("value1".to_string()));

        // List value check
        let len = Llen
            .run(vec_strings!["key2"], &mut restored_database)
            .unwrap();
        assert_eq!(len, ":4\r\n".to_string());

        let first_value = LPop
            .run(vec_strings!["key2"], &mut restored_database)
            .unwrap();
        assert_eq!(first_value, "$6\r\nvalue1\r\n".to_string());

        let last_value = RPop
            .run(vec_strings!["key2"], &mut restored_database)
            .unwrap();
        assert_eq!(last_value, "$6\r\nvalue4\r\n".to_string());

        let value_2 = LPop
            .run(vec_strings!["key2"], &mut restored_database)
            .unwrap();
        assert_eq!(value_2, "$6\r\nvalue2\r\n".to_string());

        let value_3 = LPop
            .run(vec_strings!["key2"], &mut restored_database)
            .unwrap();
        assert_eq!(value_3, "$6\r\nvalue3\r\n".to_string());

        // Set value check
        assert_eq!(
            Sismember
                .run(vec_strings!["key3", "value1"], &mut restored_database)
                .unwrap(),
            RInteger::encode(1)
        );
    }
}
