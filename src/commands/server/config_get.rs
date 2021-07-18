use crate::{
    commands::{check_empty, Runnable},
    native_types::{ErrorStruct, RArray, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
    vec_strings,
};

pub struct ConfigGet;

impl Runnable<ServerRedisAtributes> for ConfigGet {
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
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
