use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{commands::Runnable, native_types::ErrorStruct};

pub struct ConfigSetDbFileName;

impl Runnable<ServerRedisAttributes> for ConfigSetDbFileName {
    /// Rename archive file to save to database.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if CONFIG SET DBFILENAME was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty.
    /// * [ServerRedisAtributes] has poisoned methods.
    fn run(
        &self,
        _buffer: Vec<String>,
        _server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        Ok("+TODO ConfigSetDbFileName\r\n".to_string())
    }
}
