use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::{
        check_empty,
        server::{config_get::ConfigGet, config_set::ConfigSet},
        Runnable,
    },
    native_types::ErrorStruct,
};

pub struct Config;

impl Runnable<ServerRedisAttributes> for Config {
    /// Return the number of keys in the currently-selected database.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger]: a number of keys in the currently-selected database.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty.
    /// * [ServerRedisAtributes] has poisoned methods.
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config")?;

        let item = buffer.remove(0);
        match item.as_str() {
            "set" => ConfigSet.run(buffer, server),
            "get" => ConfigGet.run(buffer, server),
            _ => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("Unknown subcommand or wrong number of arguments for 'config'"),
            )),
        }
    }
}
