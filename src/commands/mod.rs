use std::sync::{
    atomic::AtomicBool,
    mpsc::{self, Receiver},
    Arc,
};

use crate::{
    communication::log_messages::LogMessage,
    messages::redis_messages,
    native_types::ErrorStruct,
    tcp_protocol::{notifier::Notifier, RawCommand},
};

pub mod keys;
pub mod lists;
pub mod pubsub;
pub mod server;
pub mod sets;
pub mod strings;

#[macro_export]
macro_rules! vec_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

#[macro_export]
macro_rules! err_wrongtype {
    () => {
        Err(ErrorStruct::new(
            redis_messages::wrongtype().get_prefix(),
            redis_messages::wrongtype().get_message(),
        ))
    };
}

/// A trait for execute commands into elements T.
pub trait Runnable<T> {
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```
    /// use redis_rust::database::Database;
    /// use redis_rust::commands::strings::set::Set;
    /// use redis_rust::native_types::ErrorStruct;
    /// use redis_rust::commands::Runnable;
    /// use redis_rust::commands::create_notifier;
    /// use redis_rust::vec_strings;
    /// use std::sync::{Arc, Mutex};
    /// fn execute<Database>(command: &dyn Runnable<Database>,
    ///            buffer: Vec<String>,
    ///            database: &mut Database)
    ///         -> Result<String, ErrorStruct>
    /// {
    ///     command.run(buffer, database)
    /// }
    ///
    /// let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
    /// let mut database = Arc::new(Mutex::new(Database::new(notifier)));
    /// let buffer = vec_strings!["key", "value"];
    /// let object_commmand = Set;
    /// let result_received = execute(&object_commmand, buffer, &mut database);
    ///
    /// let expected_result = "+OK\r\n".to_string();
    /// assert_eq!(expected_result, result_received.unwrap());
    /// ```
    fn run(&self, buffer: Vec<String>, item: &mut T) -> Result<String, ErrorStruct>;
}

// Fun aux

pub fn get_as_integer(value: &str) -> Result<isize, ErrorStruct> {
    // value es mut porque TypeSaved::String() devuelve &mut String
    match value.parse::<isize>() {
        Ok(value_int) => Ok(value_int), // if value is parsable as pointer size integer
        Err(_) => Err(ErrorStruct::new(
            "ERR".to_string(),
            "value is not an integer or out of range".to_string(),
        )),
    }
}

// Check number of arguments

fn check_empty(buffer: &[String], name: &str) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        let message_error = redis_messages::arguments_invalid_to(name);
        return Err(ErrorStruct::new(
            message_error.get_prefix(),
            message_error.get_message(),
        ));
    }

    Ok(())
}

pub fn check_not_empty(buffer: &[String]) -> Result<(), ErrorStruct> {
    if !buffer.is_empty() {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("wrong number of arguments"),
        ))
    } else {
        Ok(())
    }
}

pub fn create_notifier() -> (
    Notifier,
    Receiver<Option<LogMessage>>,
    Receiver<Option<RawCommand>>,
) {
    let (log_snd, log_rcv) = mpsc::channel();
    let (cmd_snd, cmd_rcv) = mpsc::channel();

    (
        Notifier::new(
            log_snd,
            cmd_snd,
            Arc::new(AtomicBool::new(false)),
            "test_addr".into(),
        ),
        log_rcv,
        cmd_rcv,
    )
}

fn check_error_cases_without_elements(
    buffer: &[String],
    name_command: &str,
    elements: usize,
) -> Result<(), ErrorStruct> {
    check_empty(buffer, name_command)?;

    if buffer.len() != elements {
        let error_message = redis_messages::arguments_invalid_to(name_command);
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}
