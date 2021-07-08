use crate::native_types::ErrorStruct;
use crate::native_types::{RBulkString, RedisType};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::SystemTime;

use crate::communication::log_messages::LogMessage;
use crate::tcp_protocol::client_handler::ClientHandler;

pub struct ClientList {
    list: Vec<ClientHandler>,
    channel_register: HashMap<String, usize>,
    _log_channel: Sender<LogMessage>,
}

impl ClientList {
    pub fn new(log_channel: Sender<LogMessage>) -> ClientList {
        ClientList {
            list: Vec::new(),
            channel_register: HashMap::new(),
            _log_channel: log_channel,
        }
    }

    pub fn insert(&mut self, new_client: ClientHandler) {
        self.list.push(new_client);
    }

    pub fn remove_client(&mut self, client_addr: String) {
        self.list.retain(|x| {
            let remove = x.fields.lock().unwrap().address.to_string();
            !remove.eq(&client_addr)
        });
        println!("HOLA QUE PASA");
        let clients_detail = self
            .list
            .iter()
            .map(|x| x.get_detail())
            .collect::<Vec<String>>();

        if !clients_detail.len().eq(&0) {
            self._log_channel
                .send(LogMessage::detail_clients(clients_detail))
                .unwrap();
        }
    }

    pub fn notify_monitors(&mut self, addr: String, notification: Vec<String>) {
        self.list
            .iter_mut()
            .filter(|x| x.is_monitor_notificable())
            .for_each(|client| {
                let time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(n) => n.as_secs(),
                    Err(_) => {
                        panic!("SystemTime before UNIX EPOCH! Are we travelling to the past?")
                    }
                };
                let message_to_notify = format!("At {}: [{}] {:?}\r\n", time, &addr, notification);
                client.write_stream(RBulkString::encode(message_to_notify));
            });
    }

    pub fn send_message_to_subscriptors(
        &mut self,
        channel: String,
        message: String,
    ) -> Result<usize, ErrorStruct> {
        self.list
            .iter_mut()
            .filter(|x| x.is_subscripted_to(&channel))
            .for_each(|client| {
                client.write_stream(RBulkString::encode(String::from(&message)));
            });

        Ok(self
            .list
            .iter()
            .filter(|x| x.is_subscripted_to(&channel))
            .count())
    }

    pub fn increase_channels(&mut self, channels: Vec<String>) {
        for channel in channels.iter() {
            if let Some(counter) = self.channel_register.get_mut(channel) {
                *counter += 1;
            } else {
                self.channel_register.insert(String::from(channel), 1);
            }
        }
    }

    pub fn decrease_channels(&mut self, channels: Vec<String>) {
        for channel in channels.iter() {
            let same_channel = String::from(channel);
            if let Some(counter) = self.channel_register.get_mut(channel) {
                *counter -= 1;
                if *counter == 0 {
                    self.channel_register.remove(&same_channel);
                }
            }
        }
    }
}
