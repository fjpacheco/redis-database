use std::collections::HashMap;

use crate::{native_types::NativeTypes, redis_string::RedisString};

#[derive(Debug)]
struct Database {
    pub hashmap: HashMap<String, RedisTypes>,
    pub hashmap_commands: HashMap<String, Commands>,
}

#[derive(Debug)]
pub enum RedisTypes {
    String(RedisString),
    // Lists(RedisLists),
    // Sets(RedisSets),
}
#[derive(Debug)]
#[allow(dead_code)]
enum Commands {
    Strings,
    Lists,
    Sets,
    None,
}

impl Database {
    #[allow(dead_code)]
    fn new() -> Self {
        let mut hashmap_commands = HashMap::new();
        hashmap_commands.insert("set".to_string(), Commands::Strings);
        Database {
            hashmap: HashMap::new(),
            hashmap_commands,
        }
    }

    #[allow(dead_code)]
    fn execute(&mut self, buffer: String) -> Result<NativeTypes, NativeTypes> {
        let mut buffer_split = buffer.split_whitespace();
        let command = buffer_split.next().unwrap_or("");

        if let Some(item) = self.hashmap_commands.get(command) {
            match item {
                Commands::Strings => RedisString::run(buffer, &mut self.hashmap),
                Commands::Lists => Err(NativeTypes::new_error("ERR Command not found ")),
                Commands::Sets => Err(NativeTypes::new_error("ERR Command not found ")),
                Commands::None => Err(NativeTypes::new_error("ERR Command not found ")),
            }
        } else {
            Err(NativeTypes::new_error("ERR Command not found "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test01_set_key_with_value_in_database_return_ok_simple_string() {
        let command_complete = "set key value".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete);

        assert_eq!(
            result_received.unwrap().encode().unwrap(),
            "+OK\r\n".to_string()
        )
    }
}
