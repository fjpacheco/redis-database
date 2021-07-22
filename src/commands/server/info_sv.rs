use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RArray, RedisType},
};

pub struct InfoSv;

impl Runnable<ServerRedisAtributes> for InfoSv {
    fn run(
        &self,
        _buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        Ok(RArray::encode(server.info()?))
    }
}
