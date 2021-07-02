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
    }

    pub fn notify_monitors(&mut self, notification: Vec<String>) {
        self.list
            .iter_mut()
            .filter(|x| x.is_monitor_notifiable())
            .for_each(|client| {
                let time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(n) => n.as_secs(),
                    Err(_) => {
                        panic!("SystemTime before UNIX EPOCH! Are we travelling to the past?")
                    }
                };
                let peer = client.get_peer().unwrap().to_string();
                let message_to_notify = format!("At {}: [{}] {:?}\r\n", time, peer, notification);
                client.write_stream(message_to_notify);
            });
    }

    pub fn increase_channels(&mut self, channels: &mut Vec<String>) {
        for channel in channels.iter() {
            if let Some(counter) = self.channel_register.get_mut(channel) {
                *counter += 1;
            } else {
                self.channel_register.insert(String::from(channel), 1);
            }
        }
    }

    pub fn decrease_channels(&mut self, channels: &mut Vec<String>) {
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
