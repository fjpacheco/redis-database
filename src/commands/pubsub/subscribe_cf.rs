use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType},
};

use crate::tcp_protocol::client_atributes::client_fields::ClientFields;

use std::sync::Arc;
use std::sync::Mutex;

pub struct SubscribeCF;

impl Runnable<Arc<Mutex<ClientFields>>> for SubscribeCF {
    fn run(
        &self,
        buffer: Vec<String>,
        status: &mut Arc<Mutex<ClientFields>>,
    ) -> Result<String, ErrorStruct> {
        match status.lock().unwrap().add_subscriptions(buffer) {
            Ok(added) => Ok(RInteger::encode(added)),
            Err(error) => Err(error),
        }
    }
}
