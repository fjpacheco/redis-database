use std::sync::{Arc, Mutex};

use crate::{
    commands::{check_not_empty, Runnable},
    database::Database,
    messages::redis_messages,
    native_types::{error_severity::ErrorSeverity, ErrorStruct, RSimpleString, RedisType},
};

pub struct Save;

impl Runnable<Arc<Mutex<Database>>> for Save {
    /// Execute a take snapshot on the received [Database].
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if SAVE was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The take snapshot on the [Database] failed.
    /// * The buffer [Vec]<[String]> is received not empty.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.   
    fn run(
        &self,
        buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        check_not_empty(&buffer)?;

        match database
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "database",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .take_snapshot()
        {
            Ok(_) => Ok(RSimpleString::encode(redis_messages::ok())),
            Err(_) => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("Persistence fail"),
            )),
        }
    }
}
