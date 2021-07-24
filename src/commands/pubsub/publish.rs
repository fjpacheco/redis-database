/*use crate::native_types::RInteger;
use crate::native_types::RBulkString;
use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, redis_type::RedisType},
    tcp_protocol::client_list::ClientList,
};

use super::pop_value;

use std::sync::Arc;
use std::sync::Mutex;

pub struct Publish;

impl Runnable<Arc<Mutex<ClientList>>> for Publish {
    fn run(
        &self,
        mut buffer: Vec<String>,
        clients: &mut Arc<Mutex<ClientList>>,
    ) -> Result<String, ErrorStruct> {

        let channel = pop_value(&mut buffer, "Publish")?;
        let message = concatenate_words_of_vec(buffer);
        Ok(RInteger::encode(clients.lock().unwrap().send_message_to_subscriptors(channel, message) as isize))

    }
}

fn concatenate_words_of_vec(mut buffer: Vec<String>) -> String {
    let mut message = String::new();

    for word in buffer.iter() {
        message.push(' ');
        message.push_str(word)
    }

    message
}*/

use crate::native_types::RInteger;
use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, redis_type::RedisType},
};

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
            .unwrap()
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
