use std::collections::{HashMap, HashSet};

use crate::{
    messages::redis_messages::command_not_found_in, native_types::ErrorStruct,
    redis_types::strings::redis_strings::redis_string,
};

#[derive(Debug)]
pub struct Database {
    elements: HashMap<String, TypesSaved>,
    commands: HashMap<String, TypesComannds>,
}

#[derive(Debug, PartialEq)]
pub enum TypesSaved {
    String(String),
    Lists(Vec<String>),
    Sets(HashSet<String>), // @fjpacheco: i'm not sure !
}

#[derive(Debug)]
#[allow(dead_code)]
enum TypesComannds {
    Strings,
    Lists,
    Sets,
}

impl Default for Database {
    // Se me quejaba clippy si no...
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let commands_strings = vec!["set", "get"];
        let commands_lists: Vec<&str> = vec![];
        let commands_sets: Vec<&str> = vec![];
        let mut hashmap_commands = HashMap::new();

        commands_strings.into_iter().for_each(|item| {
            hashmap_commands.insert(item.to_string(), TypesComannds::Strings);
        });
        commands_lists.into_iter().for_each(|item| {
            hashmap_commands.insert(item.to_string(), TypesComannds::Lists);
        });
        commands_sets.into_iter().for_each(|item| {
            hashmap_commands.insert(item.to_string(), TypesComannds::Sets);
        });

        Database {
            elements: HashMap::new(),
            commands: hashmap_commands,
        }
    }

    /// Ejecuta sobre la `Database` un comando con sus argumentos especificados en un buffer de tipo `String`
    ///
    /// El resultado de [`Ok()`] es un `String` con formato codificado de `redis_type` según sea lo solicitado por el buffer.
    ///
    /// El resultado de [`Err()`] es un `ErrorStruct`. Podría aparecer errores segun los siguientes casos:
    ///
    /// 1) ...
    ///
    /// # Ejemplos
    ///
    /// Uso básico:
    ///
    /// ```
    /// use proyecto_taller_1::database::Database;
    /// let command_complete_buffer = String::from("set key value");
    /// let mut database_redis = Database::new();
    /// let result_received = database_redis.execute(command_complete_buffer);
    ///
    /// assert_eq!("+OK\r\n", result_received.unwrap())
    /// ```
    #[allow(dead_code)]
    pub fn execute(&mut self, buffer: String) -> Result<String, ErrorStruct> {
        let command = buffer.split_whitespace().next().unwrap_or("");

        if let Some(item) = self.commands.get(&command.to_lowercase()) {
            match item {
                TypesComannds::Strings => redis_string::run(buffer, &mut self.elements),
                TypesComannds::Lists => Err(ErrorStruct::new(
                    "ERR Rust-eze team".to_string(),
                    "command not implemented".to_string(),
                )),
                TypesComannds::Sets => Err(ErrorStruct::new(
                    "ERR Rust-eze team".to_string(),
                    "command not implemented".to_string(),
                )),
            }
        } else {
            let message_error = command_not_found_in(buffer);
            Err(ErrorStruct::new(
                message_error.get_prefix(),
                message_error.get_message(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::native_types::{RBulkString, RSimpleString, RedisType};

    use super::*;
    #[test]
    fn test01_execute_command_set_key_with_value_in_database_return_ok_simple_string() {
        let command_complete_buffer = "SET key value".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete_buffer);

        let excepted_result = RSimpleString::decode(&mut "+OK\r\n".to_string());
        assert_eq!(
            excepted_result.unwrap(),
            RSimpleString::decode(&mut result_received.unwrap()).unwrap()
        )
    }

    #[test]
    fn test02_execute_command_get_return_a_value() {
        let mut database_redis = Database::new();
        let command_complete_buffer_seting = "set key value".to_string();
        let _ = database_redis.execute(command_complete_buffer_seting);
        let command_complete_buffer_geting = "GET key".to_string();

        let result_received = database_redis.execute(command_complete_buffer_geting);

        let excepted_result = RBulkString::encode("value".to_string());
        assert_eq!(excepted_result, result_received.unwrap());
    }

    #[test]
    fn test03_run_not_existent_command_with_many_args_return_error_native_type() {
        let command_complete = "abc02 key value value2 arg".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete.clone());
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_result = "-ERR unknown command \'abc02\', with args beginning with: \'key\', \'value\', \'value2\', \'arg\', \r\n".to_string();
        assert_eq!(excepted_result, result_received_encoded)
    }

    #[test]
    fn test04_run_not_existent_command_with_one_args_return_error_native_type() {
        let command_complete = "abc03 key".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_result =
            "-ERR unknown command \'abc03\', with args beginning with: \'key\', \r\n".to_string();
        assert_eq!(excepted_result, result_received_encoded)
    }

    #[test]
    fn test05_run_not_existent_command_without_args_return_error_native_type() {
        let command_complete = "abc04".to_string();
        let mut database_redis = Database::new();

        let result_received = database_redis.execute(command_complete);
        let result_received_encoded = result_received.unwrap_err().get_encoded_message_complete();

        let excepted_result =
            "-ERR unknown command \'abc04\', with args beginning with: \r\n".to_string();
        assert_eq!(excepted_result, result_received_encoded)
    }
}
