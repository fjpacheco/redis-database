use crate::{
    commands::{check_error_cases_without_elements, Runnable},
    messages::redis_messages,
    native_types::RedisType,
    native_types::{ErrorStruct, RInteger},
    Database,
};
pub struct Dbsize;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};
impl Runnable<Arc<Mutex<Database>>> for Dbsize {
    /// Return the number of keys in the currently-selected database.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger]: a number of keys in the currently-selected database.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty, or not received with only one element.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.    
    fn run(
        &self,
        buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        check_error_cases_without_elements(&buffer, "dbisze", 1)?;

        Ok(RInteger::encode(database.size() as isize))
    }
}
