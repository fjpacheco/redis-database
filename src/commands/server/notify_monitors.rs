use crate::native_types::error_severity::ErrorSeverity;
use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct NotifyMonitors;

impl Runnable<ServerRedisAtributes> for NotifyMonitors {
    /// Notifies each client of the [ClientList] of the commands issued by the server.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if MONITOR was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * [ServerRedisAtributes] has poisoned methods.
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
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
