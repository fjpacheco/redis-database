use std::sync::mpsc::Sender;
use std::collections::HashMap;

use crate::tcp_protocol::client_handler::ClientHandler;
use crate::communication::log_messages::LogMessage;

pub struct ClientList {
    list: Vec<ClientHandler>,
    channel_register: HashMap<String, usize>,
    log_channel: Sender<LogMessage>,
}

impl ClientList {
    
    pub fn new(log_channel: Sender<LogMessage>) -> ClientList {

        ClientList {
            list: Vec::new(),
            channel_register: HashMap::new(),
            log_channel,
        }

    }

    pub fn insert(&mut self, new_client: ClientHandler) {
        self.list.push(new_client);
    }

    pub fn send_message(message: String, channel: String) {


    }

    pub fn notify_monitors(notification: String) {
        
    }

    pub fn increase_channels(&mut self, channels: &mut Vec<String>) {
        for channel in channels.iter() {
            if let Some(counter) = self.channel_register.get_mut(channel){
                *counter += 1;
            } else {
                self.channel_register.insert(String::from(channel), 1);
            }
        }
    }

    pub fn decrease_channels(&mut self, channels: &mut Vec<String>) {
        for channel in channels.iter() {
            let same_channel = String::from(channel);
            if let Some(counter) = self.channel_register.get_mut(channel){
                *counter -= 1;
                if *counter == 0 {
                    self.channel_register.remove(&same_channel);
                }
            } 
        }
    }

}

