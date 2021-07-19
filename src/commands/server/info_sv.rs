use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RArray, RedisType},
};

pub struct InfoSV;

impl Runnable<ServerRedisAtributes> for InfoSV {
    fn run(
        &self,
        _buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        Ok(RArray::encode(server.info()?))
    }
}
