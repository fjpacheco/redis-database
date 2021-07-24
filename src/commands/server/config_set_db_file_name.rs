use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{commands::Runnable, native_types::ErrorStruct};

pub struct ConfigSetDbFileName;

impl Runnable<ServerRedisAttributes> for ConfigSetDbFileName {
    fn run(
        &self,
        _buffer: Vec<String>,
        _server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        Ok("+TODO ConfigSetDbFileName\r\n".to_string())
    }
}
