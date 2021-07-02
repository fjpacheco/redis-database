use crate::{
    commands::Runnable,
    messages::redis_messages::wrong_number_args_for,
    native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType},
    tcp_protocol::client_list::ClientList,
};

use crate::tcp_protocol::client_atributes::client_fields::ClientFields;

use std::sync::Arc;
use std::sync::Mutex;

pub struct Subscribe;

impl Runnable<(Arc<Mutex<ClientFields>>, ClientList)> for Subscribe {
    fn run(
        &self,
        mut buffer: Vec<String>,
        (status, client_list): &mut (Arc<Mutex<ClientFields>>, ClientList),
    ) -> Result<String, ErrorStruct> {
        if !buffer.is_empty() {
            let channels_added = status.lock().unwrap().add_subscriptions(&buffer)?;
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
