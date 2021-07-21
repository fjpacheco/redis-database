use super::{no_more_values, pop_value};
use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RInteger, RedisType},
    Database,
};
use std::sync::{Arc, Mutex};
pub struct Persist;

impl Runnable<Arc<Mutex<Database>>> for Persist {
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
