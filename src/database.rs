use crate::commands::{create_notifier, server::info_formatter::info_db_formatter};
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
    fs::{self, File},
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

    pub fn set_redis_config(&mut self, redis_config: Arc<Mutex<RedisConfig>>) {
        // TODO
        self.redis_config = Some(redis_config);
    }

    pub fn new_from(
        config: Arc<Mutex<RedisConfig>>,
        notifier: Notifier,
    ) -> Result<Self, ErrorStruct> {
        let mut elements = HashMap::new();
        let file = File::open(config
            .lock()
            .map_err(|_| { ErrorStruct::from(
                redis_messages::poisoned_lock(
                    "redis config",
                    crate::native_types::error_severity::ErrorSeverity::ShutdownServer,
                ))
            })?
            .db_filename()).map_err(|_| { ErrorStruct::from(
            redis_messages::init_failed(
                "dbfile name",
                crate::native_types::error_severity::ErrorSeverity::ShutdownServer,
            ))
        })?; //Deberia devolver Result o algo asi
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
            while let Some(line) = lines.next() {
                match line {
                    Ok(line) => {
                    let expire_info = get_expire_info(line.clone(), &mut lines)?;
                    let type_decoded = decode_case(&mut lines)?;
                    let key_decoded = decode_key(&mut lines)?;
                    let value_decoded = decode_value(&mut lines, type_decoded)?;
                    elements.insert(key_decoded, (expire_info, value_decoded)); //TODO
                }
                Err(_) => {
                    return Err(ErrorStruct::new(
                        "ERR".to_string(),
                        "Some error".to_string(), // TODO
                    ));
                }
            }
        };

        Ok(Database {
            elements,
            redis_config: Some(config),
            notifier: Arc::new(Mutex::new(notifier)),
        })
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

    pub fn take_snapshot(
        &mut self,
        notifier: Option<Arc<Mutex<Notifier>>>,
    ) -> Result<(), ErrorStruct> {
        // TODO: recordar chequear el unwrap
        let mut config = self.redis_config.as_ref().unwrap().lock().unwrap();
        let mut file = config.get_mut_dump_file().unwrap();
        for (key, (expire_info, typesaved)) in self.elements.iter_mut() {
            let mut expire_clone = expire_info.clone();
            if expire_clone.is_expired(notifier.clone(), key).not() {
                let time = expire_clone.ttl().map(|t| t as isize).unwrap_or(-1);
                write_integer_to_file(time, &mut file)?;
                persist_data(&key, &mut file, typesaved)?;
            }
        }
        Ok(())
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

fn decode_value(
    //TODO: refactor
    lines: &mut Lines<BufReader<File>>,
    type_decoded: isize,
) -> Result<TypeSaved, ErrorStruct> {
    if let Some(line) = lines.next() {
        match line {
            Ok(mut line) => {
                //TODO: refactor
                match type_decoded {
                    0 => {
                        check_decodable_line(&mut line, '+')?;
                        let value = match RSimpleString::decode(line, lines) {
                            Ok(value) => value,
                            Err(err) => return Err(err),
                        };
                        return Ok(TypeSaved::String(value));
                    }
                    1 => {
                        check_decodable_line(&mut line, '*')?;
                        let value = match RArray::decode(line, lines) {
                            Ok(value) => value,
                            Err(err) => return Err(err),
                        };
                        return Ok(TypeSaved::List(VecDeque::from(value))); // chequear si el orden del vecdeque es correcto
                    }
                    _ => {
                        check_decodable_line(&mut line, '*')?;
                        let value = match RArray::decode(line, lines) {
                            Ok(value) => value,
                            Err(err) => return Err(err),
                        };
                        return Ok(TypeSaved::Set(value.into_iter().collect()));
                    }
                }
            }
            Err(_) => {
                return Err(ErrorStruct::new(
                    "ERR".to_string(),
                    "Some error".to_string(), // TODO
                ));
            }
        }
    } else {
        Err(ErrorStruct::new(
            "ERR".to_string(),
            "Some error".to_string(), // TODO
        ))
    }
}

fn decode_key(lines: &mut Lines<BufReader<File>>) -> Result<String, ErrorStruct> {
    if let Some(line) = lines.next() {
        match line {
            Ok(mut line) => {
                check_decodable_line(&mut line, '+')?;
                RSimpleString::decode(line, lines)
            }
            Err(_) => {
                return Err(ErrorStruct::new(
                    "ERR".to_string(),
                    "Some error".to_string(), // TODO
                ));
            }
        }
    } else {
        Err(ErrorStruct::new(
            "ERR".to_string(),
            "Some error".to_string(), // TODO
        ))
    }
}

fn decode_case(lines: &mut Lines<BufReader<File>>) -> Result<isize, ErrorStruct> {
    if let Some(line) = lines.next() {
        match line {
            Ok(mut line) => {
                check_decodable_line(&mut line, ':')?;
                get_case(line, lines)
            }
            Err(_) => {
                return Err(ErrorStruct::new(
                    "ERR".to_string(),
                    "Some error".to_string(), // TODO
                ));
            }
        }
    } else {
        Err(ErrorStruct::new(
            "ERR".to_string(),
            "Some error".to_string(), // TODO
        ))
    }
}

fn get_case(line: String, lines: &mut Lines<BufReader<File>>) -> Result<isize, ErrorStruct> {
    let value = RInteger::decode(line, lines)?;
    if value == 0 || value == 1 || value == 2 {
        return Ok(value);
    }
    Err(ErrorStruct::new(
        "ERR".to_string(),
        "Some error".to_string(), // TODO
    ))
}

fn get_expire_info(
    line: String,
    lines: &mut Lines<BufReader<File>>,
) -> Result<ExpireInfo, ErrorStruct> {
    let mut line = line.clone();
    check_decodable_line(&mut line, ':')?;
    let ttl_decoded = RInteger::decode(line.clone(), lines)?;
    println!("ttl: {}", ttl_decoded);
    let mut expire_info: ExpireInfo = ExpireInfo::new();
    if ttl_decoded >= 0 {
        expire_info.set_timeout(ttl_decoded as u64)?;
    }
    Ok(expire_info)
}

fn check_decodable_line(line: &mut String, char: char) -> Result<(), ErrorStruct> {
    if
    /*line.pop().unwrap() != '\r' ||*/
    line.remove(0) != char {
        return Err(ErrorStruct::new(
            "ERR".to_string(),
            "Some error".to_string(), // TODO
        ));
    }
    Ok(())
}

fn write_string_to_file(string: &String, file: &mut File) -> Result<(), ErrorStruct> {
    if let Ok(_) = file.write_all(RSimpleString::encode(string.clone()).as_bytes()) {
        Ok(())
    } else {
        Err(ErrorStruct::new(
            "ERR_WRITE".to_string(),
            "File write failed".to_string(),
        ))
    }
}

fn write_integer_to_file(number: isize, file: &mut File) -> Result<(), ErrorStruct> {
    if let Ok(_) = file.write_all(RInteger::encode(number).as_bytes()) {
        Ok(())
    } else {
        Err(ErrorStruct::new(
            "ERR_WRITE".to_string(),
            "File write failed".to_string(),
        ))
    }
}

fn write_array_to_file(vector: Vec<String>, file: &mut File) -> Result<(), ErrorStruct> {
    if let Ok(_) = file.write_all(RArray::encode(vector).as_bytes()) {
        Ok(())
    } else {
        Err(ErrorStruct::new(
            "ERR_WRITE".to_string(),
            "File write failed".to_string(),
        ))
    }
}

enum TypeCase {
    String = 0,
    List = 1,
    Set = 2,
}

fn persist_data(key: &String, file: &mut File, typesaved: &TypeSaved) -> Result<(), ErrorStruct> {
    match typesaved {
        TypeSaved::String(value) => {
            write_integer_to_file(TypeCase::String as isize, file)?; // 0: String Encoding
            write_string_to_file(key, file)?; // KEY encoded as Redis String
            write_string_to_file(&value, file)?;
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
/*
#[cfg(test)]
mod test_database {

    use crate::{
        commands::{
            create_notifier,
            lists::rpush::RPush,
            sets::{sadd::Sadd, sismember::Sismember},
            Runnable,
        },
        vec_strings,
    };

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

    #[test]
    fn test07_persist_string_values_at_file() {
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

        original_database.take_snapshot(None).unwrap();

        assert_eq!(
            fs::read("database_01.rdb").unwrap(),
            b":-1\r\n:0\r\n+key1\r\n+value1\r\n"
        );
    }

    #[test]
    fn test08_persist_expirable_string_values_at_file() {
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

        database.take_snapshot(None).unwrap();

        assert_eq!(
            fs::read("database_08.rdb").unwrap(),
            b":4\r\n:0\r\n+key\r\n+value\r\n"
        );
    }

    #[test]
    fn test09_persist_list_values_at_file() {
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
        let mut database = Database::new(notifier);
        database.set_redis_config(config);
        let buffer = vec_strings!["key", "value1", "value2", "value3", "value4"];
        RPush.run(buffer, &mut database).unwrap();
        database.take_snapshot(None).unwrap();

        assert_eq!(
            fs::read("database_09.rdb").unwrap(),
            b":-1\r\n:1\r\n+key\r\n*4\r\n$6\r\nvalue1\r\n$6\r\nvalue2\r\n$6\r\nvalue3\r\n$6\r\nvalue4\r\n"
        );
    }

    #[ignore]
    #[test]
    fn test10_persist_set_values_at_file() {
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
        let mut database = Database::new(notifier);
        database.set_redis_config(config);
        let buffer = vec_strings!["key", "value1", "value2"];
        Sadd.run(buffer, &mut database).unwrap();
        database.take_snapshot(None).unwrap();

        assert_eq!(
            fs::read("database_10.rdb").unwrap(),
            b":0\r\n:2\r\n+key\r\n*2\r\n$6\r\nvalue1\r\n$6\r\nvalue2\r\n"
        );
    }

    #[test]
    fn test11_restore_string_values_from_file() {
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
        let mut original_database = Database::new(notifier);
        original_database.set_redis_config(config1);
        original_database.insert(
            "key1".to_string(),
            TypeSaved::String(String::from("value1")),
        );
        original_database.take_snapshot(None).unwrap();

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

        let mut restored_database = Database::new_from(filename1).unwrap();
        restored_database.set_redis_config(config2);
        restored_database.take_snapshot(None).unwrap();

        assert_eq!(
            fs::read("database_11_a.rdb").unwrap(),
            fs::read("database_11_b.rdb").unwrap()
        );
    }

    #[test]
    fn test12_restore_list_values_from_file() {
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
        let mut database = Database::new(notifier);
        database.set_redis_config(config1);
        let buffer = vec_strings!["key", "value1", "value2", "value3", "value4"];
        RPush.run(buffer, &mut database).unwrap();

        database.take_snapshot(None).unwrap();

        let filename2 = "database_12_b.rdb";
        let mut restored_database = Database::new_from(filename1).unwrap();

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

        restored_database.take_snapshot(None).unwrap();

        assert_eq!(
            fs::read("database_12_a.rdb").unwrap(),
            fs::read("database_12_b.rdb").unwrap()
        );
    }

    // #[ignore] // El orden de las keys de los sets no es siempre el mismo
    #[test]
    fn test13_restore_set_values_from_file() {
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
        let mut database = Database::new(notifier);
        database.set_redis_config(config1);

        let buffer: Vec<String> = vec_strings!["key", "value1", "value2"];
        Sadd.run(buffer, &mut database).unwrap();

        database.take_snapshot(None).unwrap();

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

        let mut restored_database = Database::new_from(filename1).unwrap();
        restored_database.set_redis_config(config2);
        restored_database.take_snapshot(None).unwrap();

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
    fn test14_restore_expirable_string_values_from_file() {
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
        let mut original_database = Database::new(notifier);
        original_database.set_redis_config(config1);

        original_database.insert("key".to_string(), TypeSaved::String(String::from("value")));
        original_database.set_ttl("key", 5).unwrap();

        original_database.take_snapshot(None).unwrap();

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
        let mut restored_database = Database::new_from(filename1).unwrap();
        restored_database.set_redis_config(config2);

        restored_database.take_snapshot(None).unwrap();

        assert_eq!(
            fs::read("database_14_a.rdb").unwrap(),
            fs::read("database_14_b.rdb").unwrap()
        );
    }
}
*/