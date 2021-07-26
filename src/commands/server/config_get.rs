use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::{check_empty, Runnable},
    native_types::{ErrorStruct, RArray, RedisType},
    vec_strings,
};

pub struct ConfigGet;

impl Runnable<ServerRedisAttributes> for ConfigGet {
    /// The CONFIG GET command is used to read the configuration parameters of a running Redis server
    ///
    /// # Return value
    /// [String] _encoded_ in [RArray]: specifically:
    /// * port: accept connections on the specified port.
    /// * timeout: close the connection after a client is idle for N seconds.
    /// * logfile: specify the log file name.
    /// * dbfilename: specify the dbfile name.
    /// * verbose: level for visualization information.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty.
    /// * [ServerRedisAttributes](crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes) has poisoned methods.
    /// * Unknown subcommand for CONFIG GET.
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config get")?;

        let item = buffer.remove(0);
        match item.as_str() {
            "port" => Ok(RArray::encode(vec_strings!("port", server.get_port()?))),
            "timeout" => Ok(RArray::encode(vec_strings!(
                "timeout",
                server.get_timeout()?
            ))),
            "logfile" => Ok(RArray::encode(vec_strings!(
                "logfile",
                server.get_logfile_name()?
            ))),
            "dbfilename" => Ok(RArray::encode(vec_strings!(
                "dbfilename",
                server.get_dbfile_name()?
            ))),
            "verbose" => Ok(RArray::encode(vec_strings!(
                "verbose",
                server.get_verbose()?
            ))),
            _ => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("Unknown subcommand or wrong number of arguments for 'get'."),
            )),
        }
    }
}
