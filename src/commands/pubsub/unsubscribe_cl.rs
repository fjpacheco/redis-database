use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::Runnable,
    native_types::{ErrorStruct, RBulkString, RedisType},
};

/// Remove the given channels to the channels register.
///
/// # Return value
/// [String] encoding a [isize]: the number of channels that have
/// been removed.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// * The list's lock is poisoned.
pub struct UnsubscribeCl;

impl Runnable<ServerRedisAttributes> for UnsubscribeCl {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        server
            .get_client_list()
            .lock()
            .unwrap()
            .decrease_channels(buffer);
        Ok(RBulkString::encode("".to_string()))
    }
}
