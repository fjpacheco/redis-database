use crate::{
    commands::{pubsub::no_more_values, Runnable},
    native_types::{ErrorStruct, RArray, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
};

pub struct Numsub;

impl Runnable<ServerRedisAtributes> for Numsub {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        no_more_values(&buffer, "numsub")?;
        Ok(RArray::encode(
            server.get_client_list().lock().unwrap().get_register(),
        ))
    }
}
