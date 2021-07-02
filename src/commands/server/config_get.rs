use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
    vec_strings,
};

pub struct ConfigGet;

impl Runnable<ServerRedisAtributes> for ConfigGet {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        if buffer.len() != 1 {
            let error_message = redis_messages::arguments_invalid_to("config get");
            return Err(ErrorStruct::new(
                error_message.get_prefix(),
                error_message.get_message(),
            ));
        }

        let item = buffer.get(0).unwrap();
        match item.as_str() {
            "timeout" => Ok(RArray::encode(vec_strings!(
                "timeout",
                server.get_timeout()
            ))),
            "port" => Ok(RArray::encode(vec_strings!("port", server.get_port()))),
            _ => Ok(RArray::encode(vec![])),
        }
    }
}
