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
        mut buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        let addr = buffer.pop();
        server
            .shared_clients
            .lock()
            .unwrap()
            .notify_monitors(addr.unwrap(), buffer);
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
