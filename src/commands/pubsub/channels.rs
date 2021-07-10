use crate::{
    commands::{
        pubsub::{no_more_values, pop_value},
        Runnable,
    },
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RedisType},
    regex::super_regex::SuperRegex,
    tcp_protocol::server::ServerRedisAtributes,
};

pub struct Channels;

impl Runnable<ServerRedisAtributes> for Channels {
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        let pattern = pop_value(&mut buffer, "channels")?;
        no_more_values(&buffer, "channels")?;

        let regex = match SuperRegex::from(&pattern) {
            Ok(matcher) => matcher,
            Err(_) => {
                return Err(ErrorStruct::from(redis_messages::wrong_regex_pattern(
                    &pattern,
                )));
            }
        };

        Ok(RArray::encode(
            server
                .get_client_list()
                .lock()
                .unwrap()
                .match_pattern(regex),
        ))
    }
}
