use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::Runnable,
    database::Database,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};
use std::sync::{Arc, Mutex};
pub struct Save;

impl Runnable<Arc<Mutex<Database>>> for Save {
    fn run(
        &self,
        _buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let _database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        //

        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
