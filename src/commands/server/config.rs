use crate::{
    commands::{
        check_empty,
        server::{config_get::ConfigGet, config_set::ConfigSet},
        Runnable,
    },
    native_types::ErrorStruct,
    tcp_protocol::server::ServerRedisAtributes,
};

pub struct Config;

impl Runnable<ServerRedisAtributes> for Config {
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
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
