use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{
    commands::Runnable,
    native_types::{ErrorStruct, RBulkString, RedisType},
};

pub struct SubscribeCl;

impl Runnable<ServerRedisAtributes> for SubscribeCl {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        server
            .get_client_list()
            .lock()
            .unwrap()
            .increase_channels(buffer);
        Ok(RBulkString::encode("".to_string()))
    }
}
