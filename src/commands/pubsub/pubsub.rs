
use crate::{
    commands::{
        Runnable,
        pubsub::{
            pop_value,
            channels::Channels,
            numsub::Numsub,
        }
    },
    native_types::{ErrorStruct},
    tcp_protocol::server::ServerRedisAtributes,
    messages::redis_messages,
};

pub struct Pubsub;

impl Runnable<ServerRedisAtributes> for Pubsub {
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {

        let mut subcommand = pop_value(&mut buffer, "pubsub")?;
        subcommand.make_ascii_lowercase();
        match subcommand.as_str() {
            "channels" => Channels.run(buffer, server),
            "numsub" => Numsub.run(buffer, server),
            _ => Err(ErrorStruct::from(redis_messages::unknown_command(subcommand, buffer))),
        }

    }

}