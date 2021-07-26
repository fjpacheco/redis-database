use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::{check_empty, Runnable},
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct ConfigSetLogFile;

impl Runnable<ServerRedisAttributes> for ConfigSetLogFile {
    /// Change the name of the log file used to store debug information.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if CONFIG SET LOGFILE was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty.
    /// * [ServerRedisAtributes] has poisoned methods.
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config set logfile")?;

        let new_file_name = buffer.get(0).unwrap().to_string(); // no empty!
        server.change_logfilename(new_file_name)?;
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
