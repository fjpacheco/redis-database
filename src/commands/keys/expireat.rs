use super::{no_more_values, parse_integer, pop_value};
use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RInteger, RedisType},
    Database,
};
use std::sync::{Arc, Mutex};
pub struct ExpireAt;

impl Runnable<Arc<Mutex<Database>>> for ExpireAt {
    /// EXPIREAT has the same effect and semantic as EXPIRE, but instead of specifying
    /// the number of seconds representing the TTL (time to live), it takes an absolute
    /// Unix timestamp. A timestamp in the past will delete the key immediately.
    ///
    /// # Return value
    /// * [String] _encoded_ in [RInteger]: 1 if the timeout was set.
    /// * [String] _encoded_ in [RInteger]: 0 if key does not exist.
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
        let timeout = pop_value(&mut buffer, "Expire")?;
        if timeout.starts_with('-') {
            return Err(ErrorStruct::from(redis_messages::negative_number()));
        }
        let timeout = parse_integer(timeout)? as u64;
        let key = pop_value(&mut buffer, "Expire")?;
        no_more_values(&buffer, "Expire")?;

        check_errors(database.set_ttl_unix_timestamp(&key, timeout))
    }
}

// Returns an integer encoded as RInteger or an error according to the received parameter.
// If the prefix read matches "TTL", returns error. Otherwise returns 0 or 1, if key does
// not exist or if timeout was set respectively.
fn check_errors(should_be_error: Result<(), ErrorStruct>) -> Result<String, ErrorStruct> {
    if let Err(error) = should_be_error {
        if let Some(prefix) = error.prefix() {
            match prefix {
                "TTL" => Err(error),
                _ => Ok(RInteger::encode(0)),
            }
        } else {
            Ok(RInteger::encode(0))
        }
    } else {
        Ok(RInteger::encode(1))
    }
}
