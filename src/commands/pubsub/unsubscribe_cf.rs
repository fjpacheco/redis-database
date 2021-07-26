use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::error_severity::ErrorSeverity,
    native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType},
};

use crate::tcp_protocol::client_atributes::client_fields::ClientFields;

use std::sync::Arc;
use std::sync::Mutex;

/// Remove the given channels to the subscription list of the client.
///
/// # Return value
/// [String] encoding a [isize]: the number of channels that have
/// been removed.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// * The client list's lock is poisoned.
pub struct UnsubscribeCf;

impl Runnable<Arc<Mutex<ClientFields>>> for UnsubscribeCf {
    fn run(
        &self,
        buffer: Vec<String>,
        status: &mut Arc<Mutex<ClientFields>>,
    ) -> Result<String, ErrorStruct> {
        match status
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "client fields",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .remove_subscriptions(buffer)
        {
            Ok(added) => Ok(RInteger::encode(added)),
            Err(error) => Err(error),
        }
    }
}
