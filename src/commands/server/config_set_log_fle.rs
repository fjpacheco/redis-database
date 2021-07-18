use crate::{
    commands::{check_empty, Runnable},
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
};
pub struct ConfigSetLogFile;

impl Runnable<ServerRedisAtributes> for ConfigSetLogFile {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config set logfile")?;

        let new_file_name = buffer.get(0).unwrap().to_string(); // no empty!
        server.change_logfilename(new_file_name)?;
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
