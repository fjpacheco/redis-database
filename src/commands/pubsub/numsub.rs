use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::{pubsub::no_more_values, Runnable},
    native_types::{ErrorStruct, RArray, RedisType},
};

pub struct Numsub;

impl Runnable<ServerRedisAttributes> for Numsub {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        no_more_values(&buffer, "numsub")?;
        Ok(RArray::encode(
            server.get_client_list().lock().unwrap().get_register(),
        ))
    }
}
