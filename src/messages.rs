use crate::native_types::error_severity::ErrorSeverity;

pub struct MessageRedis {
    prefix: String,
    message: String,
    severity: ErrorSeverity,
}

impl MessageRedis {
    /// Prefix getter
    pub fn get_prefix(&self) -> String {
        self.prefix.to_string()
    }

    /// Message getter
    pub fn get_message(&self) -> String {
        self.message.to_string()
    }

    /// Severity getter
    pub fn get_severity(&self) -> ErrorSeverity {
        self.severity.clone()
    }

    /// Complete message getter
    pub fn get_message_complete(&self) -> String {
        self.prefix.to_owned() + " " + &self.message
    }
}

pub mod redis_messages {
    use super::MessageRedis;
    use crate::native_types::error_severity::ErrorSeverity;
    use crate::native_types::ErrorStruct;

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
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn not_empty_values_for(item: &str) -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "\'".to_owned() + item + "\'" + " does not accept empty values",
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn syntax_error() -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "syntax error".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn wrong_number_args_for(item: &str) -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "wrong number of arguments for ".to_owned() + "\'" + item + "\'" + " command",
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn wrongtype() -> MessageRedis {
        MessageRedis {
            prefix: "WRONGTYPE".to_string(),
            message: "Operation against a key holding the wrong kind of value".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn negative_number() -> MessageRedis {
        MessageRedis {
            prefix: "NEG".to_string(),
            message: "Given number must be positive or zero".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn key_not_found() -> MessageRedis {
        MessageRedis {
            prefix: "KEYNOTFOUND".to_string(),
            message: "Session does not exist or has timed out".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn write_error() -> MessageRedis {
        MessageRedis {
            prefix: "ERR_WRITE".to_string(),
            message: "File write failed".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn ttl_epoch_error() -> MessageRedis {
        MessageRedis {
            prefix: "TTL".to_string(),
            message: "an error occurred with the epoch expiration".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn cannot_write_stream() -> MessageRedis {
        MessageRedis {
            prefix: "CANNOTWRITE".to_string(),
            message: "an error occurred while writing the tcp stream".to_string(),
            severity: ErrorSeverity::CloseClient,
        }
    }

    pub fn client_timeout_expired() -> MessageRedis {
        MessageRedis {
            prefix: "CLIENT_OFF".to_string(),
            message: "timeout expired".to_string(),
            severity: ErrorSeverity::CloseClient,
        }
    }

    pub fn not_valid_executor() -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "Executor can't execute this command".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn not_valid_pubsub() -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message:
                "can't execute command: only SUBSCRIBE and UNSUBSCRIBE are allowed in this context"
                    .to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn not_valid_monitor() -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "Replica can't interract with the keyspace".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn unexpected_behaviour(reason: &str) -> MessageRedis {
        MessageRedis {
            prefix: "INSTAPANIC".to_string(),
            message: reason.to_string(),
            severity: ErrorSeverity::ShutdownServer,
        }
    }

    pub fn closed_socket() -> MessageRedis {
        MessageRedis {
            prefix: "SOCKET".to_string(),
            message: "Attempted to write to a closed socket".to_string(),
            severity: ErrorSeverity::CloseClient,
        }
    }

    pub fn clone_socket() -> MessageRedis {
        MessageRedis {
            prefix: "SOCKET".to_string(),
            message: "Clone attempt failed".to_string(),
            severity: ErrorSeverity::CloseClient,
        }
    }

    pub fn closed_sender(severity: ErrorSeverity) -> MessageRedis {
        MessageRedis {
            prefix: "SENDER".to_string(),
            message: "Attempted to send to a closed channel".to_string(),
            severity,
        }
    }

    pub fn init_failed(process: &str, severity: ErrorSeverity) -> MessageRedis {
        MessageRedis {
            prefix: "INITFAILED".to_string(),
            message: format!("Failed to create {}", process),
            severity,
        }
    }

    pub fn wrong_regex_pattern(regex: &str) -> MessageRedis {
        MessageRedis {
            prefix: "REGEX".to_string(),
            message: format!("Given pattern could not be parsed: {}", regex),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn poisoned_lock(lock: &str, severity: ErrorSeverity) -> MessageRedis {
        MessageRedis {
            prefix: "POISONLOCK".to_string(),
            message: format!("Failed to access to lock: {}", lock),
            severity,
        }
    }

    pub fn thread_panic(name_thread: &str) -> MessageRedis {
        MessageRedis {
            prefix: "THREADPANIC".to_string(),
            message: format!("Thread '{}' has panicked", name_thread),
            severity: ErrorSeverity::ShutdownServer,
        }
    }

    pub fn empty_buffer() -> MessageRedis {
        MessageRedis {
            prefix: "EMPTYBUFFER".to_string(),
            message: "Failed to read a client socket buffer".to_string(),
            severity: ErrorSeverity::CloseClient,
        }
    }

    pub fn broken_state() -> MessageRedis {
        MessageRedis {
            prefix: "BROKENFIELDS".to_string(),
            message: "Client's fields are in a broken state".to_string(),
            severity: ErrorSeverity::CloseClient,
        }
    }

    pub fn file_read_error() -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "Something interrupted file read".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn normal_error() -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: "Some command could not be run".to_string(),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn maximum_amount_exceeded(max: usize) -> MessageRedis {
        MessageRedis {
            prefix: "ERR".to_string(),
            message: format!(
                "There is no command which name has more than {} characters\r\n",
                max
            ),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn unknown_command(command_type: String, buffer: Vec<String>) -> MessageRedis {
        let mut args_received = String::new();
        buffer
            .into_iter()
            .for_each(|one_arg| args_received.push_str(&("\'".to_owned() + &one_arg + "\', ")));

        MessageRedis {
            prefix: "UNKNOWN".to_string(),
            message: format!(
                "unknown command \'{}\', with args beginning with: {}",
                command_type, args_received
            ),
            severity: ErrorSeverity::Comunicate,
        }
    }

    pub fn command_not_found(command_type: String, buffer: Vec<String>) -> ErrorStruct {
        let mut args_received = String::new();
        buffer
            .into_iter()
            .for_each(|one_arg| args_received.push_str(&("\'".to_owned() + &one_arg + "\', ")));

        ErrorStruct::new(
            "ERR".to_string(),
            format!(
                "unknown command \'{}\', with args beginning with: {}",
                command_type, args_received
            ),
        )
    }

    pub fn redis_logo(port: &str) -> String {
        /*" ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠿⠿⠿⠿⠿⠿⣿⣿⣿⣿⣿⣿⣿⣿\n".to_owned()
        + " ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⣉⣁⣤⣤⣶⣾⣿⣿⣶⡄⢲⣯⢍⠁⠄⢀⢹⣿\n"
        + " ⣿⣿⣿⣿⣿⣿⣿⣿⣿⢯⣾⣿⣿⣏⣉⣹⠿⠇⠄⠽⠿⢷⡈⠿⠇⣀⣻⣿⡿⣻\n"
        + " ⣿⣿⡿⠿⠛⠛⠛⢛⡃⢉⢣⡤⠤⢄⡶⠂⠄⠐⣀⠄⠄⠄⠄⠄⡦⣿⡿⠛⡇⣼\n"
        + " ⡿⢫⣤⣦⠄⠂⠄⠄⠄⠄⠄⠄⠄⠄⠠⠺⠿⠙⠋⠄⠄⠄⠢⢄⠄⢿⠇⠂⠧⣿\n"
        + " ⠁⠄⠈⠁⠄⢀⣀⣀⣀⣀⣠⣤⡤⠴⠖⠒⠄⠄⠄⠄⠄⠄⠄⠄⠄⠘⢠⡞⠄⣸\n"
        + " ⡀⠄⠄⠄⠄⠄⠤⠭⠦⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⣂⣿\n"
        + " ⣷⡀⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⢳⠄⠄⢀⠈⣠⣤⣤⣼⣿\n"
        + " ⣿⣿⣷⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣴⣶⣶⣶⣄⡀⠄⠈⠑⢙⣡⣴⣿⣿⣿⣿⣿\n"
        + " ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿\n"
        + */"                _._                                                  \n".to_owned()
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
            + " |    `-._`-._        _.-'_.-'    |     https://github.com/taller-1-fiuba-rust/Rust-eze\n"
            + "  `-._    `-._`-.__.-'_.-'    _.-'                                   \n"
            + " |`-._`-._    `-.__.-'    _.-'_.-'|                                  \n"
            + " |    `-._`-._        _.-'_.-'    |                                  \n"
            + "  `-._    `-._`-.__.-'_.-'    _.-'                                   \n"
            + "      `-._    `-.__.-'    _.-'                                       \n"
            + "          `-._        _.-'                                           \n"
            + "              `-.__.-'                                               \n\n"
    }
}
