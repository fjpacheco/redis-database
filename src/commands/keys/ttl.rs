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
pub struct Ttl;

impl Runnable<Arc<Mutex<Database>>> for Ttl {
    /// Returns the remaining time to live of a key that has a timeout. This capability allows a
    /// Redis client to check how many seconds a given key will continue to be part of the dataset.
    /// The command returns -2 if the key does not exist.
    /// The command returns -1 if the key exists but has no associated expire.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): TTL in seconds, or a negative value in order to signal an error.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty, or received with more than 1 element.
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
        let key = pop_value(&mut buffer, "Ttl")?;
        no_more_values(&buffer, "Ttl")?;

        if database.contains_key(&key) {
            if let Some(ttl) = database.ttl(&key) {
                Ok(RInteger::encode(ttl as isize))
            } else {
                Ok(RInteger::encode(-1))
            }
        } else {
            Ok(RInteger::encode(-2))
        }
    }
}
