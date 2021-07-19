use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RArray, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
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
