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
