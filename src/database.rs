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
}

impl Database {
    #[allow(dead_code)]
    fn new() -> Self {
        let mut hashmap_commands= HashMap::new();

        let commands_strings = vec!["set"].iter().map(|s| s.to_string()).collect::<Vec<String>>();
        //let commands_lists = vec!["set"].iter().map(|s| s.to_string()).collect::<Vec<String>>();
        //let commands_sets = vec!["set"].iter().map(|s| s.to_string()).collect::<Vec<String>>();

        commands_strings.into_iter().for_each(|item| {hashmap_commands.insert(item,Commands::Strings); ()} );
        //commands_lists.into_iter().for_each(|item| {hashmap_commands.insert(item,Commands::Strings); ()} );
        //commands_sets.into_iter().for_each(|item| {hashmap_commands.insert(item,Commands::Strings); ()} );

        Database {
            hashmap: HashMap::new(),
            hashmap_commands,
        }
    }

    #[allow(dead_code)]
    fn execute(&mut self, buffer: String) -> Result<NativeTypes, NativeTypes> {
        let command = buffer.split_whitespace().next().unwrap_or("");

        if let Some(item) = self.hashmap_commands.get(command) {
            match item {
                Commands::Strings => RedisString::run(buffer, &mut self.hashmap),
                Commands::Lists => Err(NativeTypes::new_error("ERR Rust-eze team: command not implemented")),
                Commands::Sets => Err(NativeTypes::new_error("ERR Rust-eze team: command not implemented")),
            }
        } else {
            Err(get_error_buffer(buffer))
        }
    }
}

fn get_error_buffer(buffer: String) -> NativeTypes {
    let mut buffer_vec: Vec<&str>= buffer.split_whitespace().collect();
    let command = buffer_vec[0];
    buffer_vec.remove(0);
    let mut args_received = String::new();
    buffer_vec.into_iter().for_each(|one_arg| args_received.push_str(&("\'".to_owned() + one_arg + "\', ")));
    let message = format!("ERR unknown command \'{}\', with args beginning with: {}", command, args_received);
    NativeTypes::new_error(&message)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test01_execute_command_set_key_with_value_in_database_return_ok_simple_string() {
        let command_complete_buffer = "set key value".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete_buffer);

        let excepted_result = "+OK\r\n".to_string();
        assert_eq!(excepted_result, result_received.unwrap().encode().unwrap())
    }
    #[test]
    fn test02_run_not_existent_command_with_many_args_return_error_native_type() {
        let command_complete = "abc02 key value value2 arg".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete);

        let excepted_result = "-ERR unknown command \'abc02\', with args beginning with: \'key\', \'value\', \'value2\', \'arg\', \r\n".to_string();
        assert_eq!(excepted_result, result_received.unwrap_err().encode().unwrap())
    }

    #[test]
    fn test03_run_not_existent_command_with_one_args_return_error_native_type() {
        let command_complete = "abc03 key".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete);

        let excepted_result = "-ERR unknown command \'abc03\', with args beginning with: \'key\', \r\n".to_string();
        assert_eq!(excepted_result, result_received.unwrap_err().encode().unwrap())
    }
    
    #[test]
    fn test04_run_not_existent_command_without_args_return_error_native_type() {
        let command_complete = "abc04".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete);

        let excepted_result = "-ERR unknown command \'abc04\', with args beginning with: \r\n".to_string();
        assert_eq!(excepted_result, result_received.unwrap_err().encode().unwrap())
    }
    
}
