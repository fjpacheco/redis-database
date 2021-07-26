use crate::{
    commands::Runnable,
    database::Database,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RArray, RedisType},
};
pub struct InfoDb;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};
impl Runnable<Arc<Mutex<Database>>> for InfoDb {
    /// Required for the INFO command. Returns information and statistics about the [Database] in a format that is simple to parse by computers and easy to read by humans.
    ///
    /// # Return value
    /// [String] _encoded_ in [RArray]: as a collection of text lines.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * [ServerRedisAttributes](crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes) has poisoned methods.
    fn run(
        &self,
        _buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        Ok(RArray::encode(database.info()?))
    }
}
