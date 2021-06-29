use crate::{
    commands::Runnable,
    native_types::{error::ErrorStruct, redis_type::RedisType, integer::RInteger},
    tcp_protocol::client_list::ClientList,
    messages::redis_messages::wrong_number_args_for,
};

use crate::tcp_protocol::client_atributes::client_status::ClientStatus;

use std::sync::Arc;
use std::sync::Mutex;

pub struct Subscribe;

impl Runnable<(Arc<Mutex<ClientStatus>>, ClientList)> for Subscribe {
    fn run(&self, mut buffer: Vec<String>, (status, client_list): &mut (Arc<Mutex<ClientStatus>>, ClientList)) -> Result<String, ErrorStruct> {
        if !buffer.is_empty() {
            let channels_added = status.lock().unwrap().add_subscriptions(&mut buffer)?;
            client_list.increase_channels(&mut buffer);
            Ok(RInteger::encode(channels_added))
        } else {
            Err(ErrorStruct::new(
                wrong_number_args_for("subscribe").get_prefix(),
                wrong_number_args_for("subscribe").get_prefix(),
            ))
        }

    }

}