use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
};

pub struct NotifyMonitors;

impl Runnable<ServerRedisAtributes> for NotifyMonitors {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        server
            .shared_clients
            .lock()
            .unwrap()
            .notify_monitors(buffer);
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
