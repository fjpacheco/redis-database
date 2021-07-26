use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RArray, RedisType},
};

pub struct InfoSv;

impl Runnable<ServerRedisAttributes> for InfoSv {
    /// Required for the INFO command. Returns information and statistics about the [ServerRedisAttributes] in a format that is simple to parse by computers and easy to read by humans.
    ///
    /// # Return value
    /// [String] _encoded_ in [RArray]: as a collection of text lines.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * [ServerRedisAtributes] has poisoned methods.
    fn run(
        &self,
        _buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        Ok(RArray::encode(server.info()?))
    }
}
