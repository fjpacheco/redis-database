use crate::{
    commands::{
        pubsub::{channels::Channels, numsub::Numsub},
        Runnable,
    },
    messages::redis_messages,
    native_types::ErrorStruct,
    tcp_protocol::server_redis_atributes::ServerRedisAtributes,
};

pub struct Pubsub;

impl Runnable<ServerRedisAtributes> for Pubsub {
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        if !buffer.is_empty() {
            let mut subcommand = buffer.remove(0);
            subcommand.make_ascii_lowercase();
            match subcommand.as_str() {
                "channels" => Channels.run(buffer, server),
                "numsub" => Numsub.run(buffer, server),
                _ => Err(ErrorStruct::from(redis_messages::unknown_command(
                    subcommand, buffer,
                ))),
            }
        } else {
            Err(ErrorStruct::from(redis_messages::wrong_number_args_for(
                "pubsub",
            )))
        }
    }
}
