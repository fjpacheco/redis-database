use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{commands::Runnable, native_types::ErrorStruct};

pub struct ConfigSetDbFileName;

impl Runnable<ServerRedisAtributes> for ConfigSetDbFileName {
    fn run(
        &self,
        _buffer: Vec<String>,
        _server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        Ok("+TODO ConfigSetDbFileName\r\n".to_string())
    }
}
