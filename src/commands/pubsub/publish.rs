use crate::native_types::RInteger;
use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, redis_type::RedisType},
};
use crate::{messages::redis_messages, native_types::error_severity::ErrorSeverity};

/// Send a message to all the subscriber of
/// a given channel.
///
/// # Return value
/// [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): the number of clients that receive the message.
pub struct Publish;

impl Runnable<ServerRedisAttributes> for Publish {
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        let channel = buffer.remove(0);
        let message = concatenate_words_of_vec(buffer);
        match server
            .get_client_list()
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "client list",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .send_message_to_subscriptors(channel, message)
        {
            Ok(count) => Ok(RInteger::encode(count as isize)),
            Err(error) => Err(error),
        }
    }
}

fn concatenate_words_of_vec(buffer: Vec<String>) -> String {
    let mut message = String::new();

    for word in buffer.iter() {
        message.push_str(word);
        message.push(' ');
    }
    message
}
