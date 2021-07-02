use crate::{
    commands::Runnable,
    native_types::{ErrorStruct, RSimpleString, RedisType},
    tcp_protocol::client_atributes::status::Status,
};

pub struct Monitor;

impl Runnable<Status> for Monitor {
    fn run(&self, mut _buffer: Vec<String>, status: &mut Status) -> Result<String, ErrorStruct> {
        status.replace(Status::Monitor);
        Ok(RSimpleString::encode(
            "MODE MONITOR ACTIVATED. PRESS CRTL+C FOR EXIT".to_string(),
        ))
    }
}
