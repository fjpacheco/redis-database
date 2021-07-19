/*use crate::native_types::RBulkString;
use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, redis_type::RedisType},
    tcp_protocol::client_list::ClientList,
};


use std::sync::Arc;
use std::sync::Mutex;

pub struct SubscribeCL;

impl Runnable<Arc<Mutex<ClientList>>> for SubscribeCL {
    fn run(
        &self,
        mut buffer: Vec<String>,
        clients: &mut Arc<Mutex<ClientList>>,
    ) -> Result<String, ErrorStruct> {

        clients.lock().unwrap().increase_channels(buffer);
        Ok(RBulkString::encode("".to_string()))

    }
}*/

use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
use crate::{
    commands::Runnable,
    native_types::{ErrorStruct, RBulkString, RedisType},
};

pub struct SubscribeCL;

impl Runnable<ServerRedisAtributes> for SubscribeCL {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        server
            .get_client_list()
            .lock()
            .unwrap()
            .increase_channels(buffer);
        Ok(RBulkString::encode("".to_string()))
    }
}
