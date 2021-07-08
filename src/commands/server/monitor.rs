use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
use crate::{
    commands::Runnable,
    native_types::{ErrorStruct, RSimpleString, RedisType},
    tcp_protocol::client_atributes::status::Status,
};
use std::sync::Arc;
use std::sync::Mutex;

pub struct Monitor;

impl Runnable<Arc<Mutex<ClientFields>>> for Monitor {
    fn run(
        &self,
        mut _buffer: Vec<String>,
        status: &mut Arc<Mutex<ClientFields>>,
    ) -> Result<String, ErrorStruct> {
        status.lock().unwrap().replace_status(Status::Monitor);
        Ok(RSimpleString::encode(
            "MONITOR MODE ACTIVATED. PRESS CRTL+C FOR EXIT".to_string(),
        ))
    }
}
