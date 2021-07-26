use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::{
        check_empty,
        server::{
            config_set_db_file_name::ConfigSetDbFileName, config_set_log_fle::ConfigSetLogFile,
            config_set_verbose::ConfigSetVerbose,
        },
        Runnable,
    },
    native_types::ErrorStruct,
};

pub struct ConfigSet;

impl Runnable<ServerRedisAttributes> for ConfigSet {
    /// The CONFIG SET command is used in order to reconfigure the server at run time without the need to restart Redis.
    ///
    /// Enabled reconfigurations:
    /// * logfile
    /// * dbfilename
    /// * verbose
    ///
    /// # Return value
    /// [String] _encoded_ in [RArray](crate::native_types::RArray): OK when the configuration was set properly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty.
    /// * [ServerRedisAttributes](crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes) has poisoned methods.
    /// * Unknown subcommand for CONFIG SET.    
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config set")?;

        let item = buffer.remove(0);
        match item.as_str() {
            "logfile" => ConfigSetLogFile.run(buffer, server),
            "dbfilename" => ConfigSetDbFileName.run(buffer, server),
            "verbose" => ConfigSetVerbose.run(buffer, server),
            _ => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("Unknown subcommand or wrong number of arguments for 'set'."),
            )),
        }
    }
}
