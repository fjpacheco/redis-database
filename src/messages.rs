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

    pub fn syntax_error() -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "syntax error".to_string(),
        }
    }

    pub fn wrong_number_args_for(item: &str) -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "wrong number of arguments for ".to_owned() + "\'" + item + "\'" + " command",
        }
    }

    pub fn wrongtype() -> MessageRedis {
        MessageRedis {
            prefix: "WRONGTYPE".to_string(),
            message: "Operation against a key holding the wrong kind of value".to_string(),
        }
    }

    pub fn key_not_found() -> MessageRedis {
        MessageRedis {
            prefix: "KEYNOTFOUND".to_string(),
            message: "Session does not exist or has timed out".to_string(),
        }
    }

    pub fn ttl_error() -> MessageRedis {
        MessageRedis {
            prefix: "TTL".to_string(),
            message: "an error occurred with the epoch expiration".to_string(),
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

    pub fn redis_logo(port: &str) -> String {
        "                _._                                                  \n".to_owned()
            + "           _.-``__ ''-._                                             \n"
            + "      _.-``    `.  `_.  ''-._           Redis Rust-eze\n"
            + "  .-`` .-```.  ```\\/    _.,_ ''-._                                  \n"
            + " (    '      ,       .-`  | `,    )                                  \n"
            + " |`-._`-...-` __...-.``-._|'` _.-'|     Port: "
            + port
            + "\n"
            + " |    `-._   `._    /     _.-'    |                                  \n"
            + "  `-._    `-._  `-./  _.-'    _.-'                                   \n"
            + " |`-._`-._    `-.__.-'    _.-'_.-'|                                  \n"
            + " |    `-._`-._        _.-'_.-'    |           https://github.com/taller-1-fiuba-rust/Rust-eze\n"
            + "  `-._    `-._`-.__.-'_.-'    _.-'                                   \n"
            + " |`-._`-._    `-.__.-'    _.-'_.-'|                                  \n"
            + " |    `-._`-._        _.-'_.-'    |                                  \n"
            + "  `-._    `-._`-.__.-'_.-'    _.-'                                   \n"
            + "      `-._    `-.__.-'    _.-'                                       \n"
            + "          `-._        _.-'                                           \n"
            + "              `-.__.-'                                               \n\n"
    }
}
