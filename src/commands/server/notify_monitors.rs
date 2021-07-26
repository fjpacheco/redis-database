use crate::native_types::error_severity::ErrorSeverity;
use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct NotifyMonitors;

impl Runnable<ServerRedisAttributes> for NotifyMonitors {
    /// Notify each client with [Status::Monitor](crate::tcp_protocol::client_atributes::status::Status) of [ClientList](crate::tcp_protocol::client_list::ClientList) of the commands issued correctly by the server.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if MONITOR was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * [ServerRedisAttributes](crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes) has poisoned methods.
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        let addr = buffer.pop();
        server
            .get_client_list()
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Client List",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .notify_monitors(addr.unwrap(), buffer);
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
