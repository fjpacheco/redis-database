use super::{no_more_values, pop_value};
use crate::database::Database;
use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RInteger, RedisType},
};

use std::sync::{Arc, Mutex};
pub struct Persist;

impl Runnable<Arc<Mutex<Database>>> for Persist {
    /// Remove the existing timeout on key, turning the key from volatile (a key with
    /// an expire set) to persistent (a key that will never expire as no timeout is
    /// associated).
    ///
    /// # Return value
    /// * [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): 1 if the timeout was removed.
    /// * [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): 0 if key does not exist or does
    /// not have an associated timeout.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty, or received with more than 1 element
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        mut buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        let key = pop_value(&mut buffer, "Persist")?;
        no_more_values(&buffer, "Persist")?;

        if database.persist(&key).is_some() {
            Ok(RInteger::encode(1))
        } else {
            Ok(RInteger::encode(0))
        }
    }
}
