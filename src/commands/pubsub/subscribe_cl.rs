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

use crate::{
    commands::Runnable,
    native_types::{ErrorStruct, RBulkString, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
};

pub struct Subscribe;

impl Runnable<ServerRedisAtributes> for Subscribe {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        server
            .shared_clients
            .lock()
            .unwrap()
            .increase_channels(buffer);
        Ok(RBulkString::encode("".to_string()))
    }
}
