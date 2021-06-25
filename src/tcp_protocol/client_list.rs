use std::sync::mpcs::Sender;

use crate::tcp_protocol::client_handler::ClientHandler;
use crate::communication::log_messages::LogMessage;

pub struct ClientList {
    list: Vec<ClientHandler>,
    log_channel: Sender<LogMessage>,
}

impl ClientList {
    
    pub fn new(log_channel: Sender<LogMessage>) -> ClientList {

        ClientList {
            list: Vec::new(),
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

}

