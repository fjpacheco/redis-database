use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::{
        error::ErrorStruct, error_severity::ErrorSeverity, integer::RInteger, redis_type::RedisType,
    },
};

use crate::tcp_protocol::client_atributes::client_fields::ClientFields;

use std::sync::Arc;
use std::sync::Mutex;

/// Add the given channels to the subscription list of the client.
///
/// # Return value
/// [String] encoding a [isize]: the number of channels that have
/// been added.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// * The client's lock is poisoned.
pub struct SubscribeCf;

impl Runnable<Arc<Mutex<ClientFields>>> for SubscribeCf {
    fn run(
        &self,
        buffer: Vec<String>,
        status: &mut Arc<Mutex<ClientFields>>,
    ) -> Result<String, ErrorStruct> {
        match status
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "status",
                    ErrorSeverity::CloseClient,
                ))
            })?
            .add_subscriptions(buffer)
        {
            Ok(added) => Ok(RInteger::encode(added)),
            Err(error) => Err(error),
        }
    }
}
