use crate::{
    commands::check_empty,
    messages::redis_messages,
    native_types::{RSimpleString, RedisType},
    tcp_protocol::server_redis_attributes::ServerRedisAttributes,
};
use crate::{commands::Runnable, native_types::ErrorStruct};

pub struct ConfigSetDbFileName;

impl Runnable<ServerRedisAttributes> for ConfigSetDbFileName {
    /// Rename archive file to save to database.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString](crate::native_types::RSimpleString): OK if CONFIG SET DBFILENAME was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty.
    /// * [ServerRedisAttributes](crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes) has poisoned methods.
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config set dbfile")?;

        let new_file_name = buffer.get(0).unwrap().to_string(); // no empty!
        server.change_dump_filename(new_file_name)?;
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
