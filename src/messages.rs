pub struct MessageRedis {
    prefix: String,
    message: String,
}

impl MessageRedis {
    pub fn get_prefix(&self) -> String {
        self.prefix.to_string()
    }
    pub fn get_message(&self) -> String {
        self.message.to_string()
    }

    // Para tests... investigar si existe una macro así: #[metodo_para_test]
    pub fn get_message_complete(&self) -> String {
        self.prefix.to_owned() + " " + &self.message
    }
}

pub mod redis_messages {
    use super::MessageRedis;

    pub fn ok() -> String {
        String::from("OK")
    }

    pub fn nil() -> String {
        String::from("(nil)")
    }

    pub fn arguments_invalid_to(item: &str) -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "wrong number of arguments for ".to_owned() + "\'" + item + "\'" + " command",
        }
    }

    pub fn not_empty_values_for(item: &str) -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "\'".to_owned() + item + "\'" + " does not accept empty values",
        }
    }

    pub fn wrong_number_args_for(item: &str) -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "wrong number of arguments for ".to_owned() + "\'" + item + "\'" + " command",
        }
    }

    pub fn wrongtype_in_get_key() -> MessageRedis {
        MessageRedis {
            prefix: "WRONGTYPE".to_string(),
            message: "Operation against a key holding the wrong kind of value".to_string(),
        }
    }

    pub fn command_not_found_in(buffer: String) -> MessageRedis {
        let mut buffer_vec: Vec<&str> = buffer.split_whitespace().collect();
        let command = buffer_vec[0];
        buffer_vec.remove(0);
        let mut args_received = String::new();
        buffer_vec
            .into_iter()
            .for_each(|one_arg| args_received.push_str(&("\'".to_owned() + one_arg + "\', ")));
        MessageRedis {
            prefix: "ERR".to_string(),
            message: format!(
                "unknown command \'{}\', with args beginning with: {}",
                command, args_received
            ),
        }
    }
}
