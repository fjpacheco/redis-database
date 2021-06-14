use crate::{messages::redis_messages, native_types::ErrorStruct};

pub mod keys;
pub mod lists;
pub mod server;
pub mod sets;
pub mod strings;

fn check_empty(buffer: &&mut Vec<&str>, name: &str) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        let message_error = redis_messages::not_empty_values_for(name);
        return Err(ErrorStruct::new(
            message_error.get_prefix(),
            message_error.get_message(),
        ));
    }

    Ok(())
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

/// A trait for execute commands into Database.
pub trait Runnable<T> {
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```
    /// use proyecto_taller_1::database::Database;
    /// use proyecto_taller_1::commands::strings::set::Set;
    /// use proyecto_taller_1::native_types::ErrorStruct;
    /// use proyecto_taller_1::commands::Runnable;
    ///
    /// fn execute<Database>(command: &dyn Runnable<Database>,
    ///            buffer: Vec<&str>,
    ///            database: &mut Database)
    ///         -> Result<String, ErrorStruct>
    /// {
    ///     command.run(buffer, database)
    /// }
    ///
    /// let mut database = Database::new();
    /// let buffer = vec!["key", "value"];
    /// let object_commmand = Set;
    /// let result_received = execute(&object_commmand, buffer, &mut database);
    ///
    /// let expected_result = "+OK\r\n".to_string();
    /// assert_eq!(expected_result, result_received.unwrap());
    /// ```
    fn run(&self, buffer_vec: Vec<&str>, database: &mut T) -> Result<String, ErrorStruct>;
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

pub fn check_not_empty(buffer: &[&str]) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("wrong number of arguments"),
        ))
    } else {
        Ok(())
    }
}

pub fn check_empty_2(buffer: &[&str]) -> Result<(), ErrorStruct> {
    if !buffer.is_empty() {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("wrong number of arguments"),
        ))
    } else {
        Ok(())
    }
}
