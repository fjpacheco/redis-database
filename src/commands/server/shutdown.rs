use std::net::TcpStream;

use crate::{commands::Runnable, native_types::ErrorStruct, tcp_protocol::server::ServerRedis};

pub struct Shutdown;

impl Runnable<ServerRedis> for Shutdown {
    fn run(&self, _buffer_vec: Vec<&str>, server: &mut ServerRedis) -> Result<String, ErrorStruct> {
        server.store(true);
        match TcpStream::connect(server.get_addr()) {
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
