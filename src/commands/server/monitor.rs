use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
use crate::{
    commands::Runnable,
    native_types::{ErrorStruct, RSimpleString, RedisType},
    tcp_protocol::client_atributes::status::Status,
};
use std::sync::Arc;
use std::sync::Mutex;

pub struct Monitor;

impl Runnable<Arc<Mutex<ClientFields>>> for Monitor {
    /// Changes the [Status] of the [ClientFields] to [Status::Monitor](crate::tcp_protocol::client_atributes::status::Status).
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: "MONITOR MODE ACTIVATED. PRESS CRTL+C FOR EXIT" if MONITOR was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * [ClientFields] received in <[Arc]<[Mutex]>> is poisoned.         
    fn run(
        &self,
        mut _buffer: Vec<String>,
        status: &mut Arc<Mutex<ClientFields>>,
    ) -> Result<String, ErrorStruct> {
        status
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Client List",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .replace_status(Status::Monitor);
        Ok(RSimpleString::encode(
            "MONITOR MODE ACTIVATED. PRESS CRTL+C FOR EXIT".to_string(),
        ))
    }
}
