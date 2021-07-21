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
pub struct Ttl;

impl Runnable<Arc<Mutex<Database>>> for Ttl {
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
