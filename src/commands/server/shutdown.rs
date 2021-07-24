use std::net::TcpStream;

use crate::messages::redis_messages;
use crate::native_types::{RSimpleString, RedisType};
use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{commands::Runnable, native_types::ErrorStruct};

pub struct Shutdown;

impl Runnable<ServerRedisAtributes> for Shutdown {
    /// The command behavior is the following:
    /// * Disconnect all the clients.
    /// * A save of the database items.
    /// * Quit the server.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if SHUTDOWN was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * [ServerRedisAtributes] has poisoned methods.
    fn run(
        &self,
        _buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        server.store(true);
        match TcpStream::connect(server.get_addr()?) {
            Ok(_) => Ok(RSimpleString::encode(redis_messages::ok())),
            Err(_) => {
                server.store(false);
                Err(ErrorStruct::new(
                    "ERR".to_string(),
                    "Error to close the server.".to_string(),
                ))
            }
        }
    }
}
