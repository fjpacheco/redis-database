use std::net::TcpStream;

use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{commands::Runnable, native_types::ErrorStruct};

pub struct Shutdown;

impl Runnable<ServerRedisAtributes> for Shutdown {
    fn run(
        &self,
        _buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        server.store(true);
        match TcpStream::connect(server.get_addr()?) {
            Ok(_) => Ok("+SERVER OFF\r\n".to_string()),
            Err(err) => {
                server.store(false);
                Err(ErrorStruct::new(
                    "ERR".to_string(),
                    format!("Error to close the server.\nDetail: {:?}", err),
                ))
            }
        }
    }
}
