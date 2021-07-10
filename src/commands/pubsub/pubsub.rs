use crate::{
    commands::{
        pubsub::{channels::Channels, numsub::Numsub, pop_value},
        Runnable,
    },
    messages::redis_messages,
    native_types::ErrorStruct,
    tcp_protocol::server::ServerRedisAtributes,
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
            _ => Err(ErrorStruct::from(redis_messages::unknown_command(
                subcommand, buffer,
            ))),
        }
    }
}
