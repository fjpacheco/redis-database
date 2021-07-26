use crate::messages::redis_messages;
use crate::{
    commands::Runnable,
    native_types::{ErrorStruct, RBulkString, RedisType},
};
use crate::{
    native_types::error_severity::ErrorSeverity,
    tcp_protocol::server_redis_attributes::ServerRedisAttributes,
};

/// Add the given channels to the channels register.
///
/// # Return value
/// [String] encoding a [isize]: the number of channels that have
/// been added.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// * The list's lock is poisoned.
pub struct SubscribeCl;

impl Runnable<ServerRedisAttributes> for SubscribeCl {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        server
            .get_client_list()
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "client list",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .increase_channels(buffer);
        Ok(RBulkString::encode("".to_string()))
    }
}
