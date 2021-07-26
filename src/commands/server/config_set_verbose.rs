use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::{check_empty, Runnable},
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct ConfigSetVerbose;

impl Runnable<ServerRedisAttributes> for ConfigSetVerbose {
    /// Change verbose level to display more or less debug information.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if CONFIG SET VERBOSE was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty.
    /// * [ServerRedisAtributes] has poisoned methods.
    /// * Invalid verbose level received.
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config set verbose")?;

        // No empty! Ok first unwrap!
        match buffer.get(0).unwrap().parse::<usize>() {
            Ok(level) => {
                server.change_verbose(level)?;
                Ok(RSimpleString::encode(redis_messages::ok()))
            }
            Err(_) => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("Invalid verbose level received."),
            )),
        }
    }
}
