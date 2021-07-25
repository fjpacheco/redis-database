use std::{
    ops::Not,
    sync::{Arc, Mutex},
};

use crate::{
    commands::{check_empty, check_empty_2, check_not_empty, Runnable},
    database::Database,
    messages::redis_messages,
    native_types::{error_severity::ErrorSeverity, ErrorStruct, RSimpleString, RedisType},
};

pub struct Save;

impl Runnable<Arc<Mutex<Database>>> for Save {
    fn run(
        &self,
        buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        check_empty_2(&buffer)?;

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
