use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
};

pub struct ClearClient;

impl Runnable<ServerRedisAtributes> for ClearClient {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        let client_addr = buffer.get(0).unwrap().to_string();
        server
            .shared_clients
            .lock()
            .unwrap()
            .remove_client(client_addr);
        println!("holaaaaaaaaaaaaaa");
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
