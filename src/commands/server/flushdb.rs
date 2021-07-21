use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RSimpleString, RedisType},
    Database,
};
pub struct FlushDB;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};
impl Runnable<Arc<Mutex<Database>>> for FlushDB {
    fn run(
        &self,
        _buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        database.clear();
        Ok(RSimpleString::encode("OK".to_string()))
    }
}
