pub mod database_mock;
use self::database_mock::DatabaseMock;
use crate::{messages::redis_messages, native_types::ErrorStruct};
pub mod lists;
pub mod sets;
pub mod strings;

// esto podria no chequearse y estar en un nivel mas arriba de jerarquia de chequeos de comandos en general.. ojo!
fn check_empty_and_name_command(buffer: &&mut Vec<&str>, name: &str) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        let message_error = redis_messages::not_empty_values_for(name);
        return Err(ErrorStruct::new(
            message_error.get_prefix(),
            message_error.get_message(),
        ));
    }
    let command_lowercase = buffer[0].to_lowercase();
    if !command_lowercase.eq(name) {
        let concat_vector_buffer = buffer.join(" ");
        let error_message = redis_messages::command_not_found_in(concat_vector_buffer);
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
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
pub trait Runnable {
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```
    /// use redis_oxidado::commands::database_mock::DatabaseMock;
    /// use redis_oxidado::commands::strings::set::Set;
    /// use redis_oxidado::native_types::ErrorStruct;
    /// use redis_oxidado::commands::Runnable;
    ///
    /// fn execute(command: &dyn Runnable,
    ///            buffer: Vec<&str>,
    ///            database: &mut DatabaseMock)
    ///         -> Result<String, ErrorStruct>
    /// {
    ///     command.run(buffer, database)
    /// }
    ///
    /// let mut database = DatabaseMock::new();
    /// let buffer = vec!["set", "key", "value"];
    /// let object_commmand = Set;
    /// let result_received = execute(&object_commmand, buffer, &mut database);
    ///
    /// let expected_result = "+OK\r\n".to_string();
    /// assert_eq!(expected_result, result_received.unwrap());
    /// ```
    fn run(
        &self,
        buffer_vec: Vec<&str>,
        database: &mut DatabaseMock,
    ) -> Result<String, ErrorStruct>;
}
