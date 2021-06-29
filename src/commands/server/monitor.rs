use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, redis_type::RedisType, simple_string::SimpleString},
    tcp_protocol::client_atributes::client_status::ClientStatus,
    
};

pub struct Monitor;

impl Runnable<Arc<Mutex<ClientStatus>>> for Monitor {
    fn run(&self, mut buffer: Vec<String>, status: &mut Arc<Mutex<ClientStatus>>) -> Result<String, ErrorStruct> {
        status.lock().unwrap().replace_status(Status::Monitor);
        Ok(SimpleString::encode("OK".to_string())
    }
}