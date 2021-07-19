use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
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
            .get_client_list()
            .lock()
            .unwrap()
            .notify_monitors(addr.unwrap(), buffer);
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
