use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RArray, RedisType},
};

pub struct InfoSv;

impl Runnable<ServerRedisAttributes> for InfoSv {
    fn run(
        &self,
        _buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        Ok(RArray::encode(server.info()?))
    }
}
