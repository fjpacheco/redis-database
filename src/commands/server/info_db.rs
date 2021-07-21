use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RArray, RedisType},
    Database,
};
pub struct InfoDB;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};
impl Runnable<Arc<Mutex<Database>>> for InfoDB {
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
