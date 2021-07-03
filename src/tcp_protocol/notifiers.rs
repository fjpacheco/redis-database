use std::{
    net::TcpStream,
    sync::{
        mpsc::{channel, Receiver, SendError, Sender},
        Arc, Mutex,
    },
};

use crate::{communication::log_messages::LogMessage, vec_strings};

use super::{client_atributes::client_fields::ClientFields, RawCommand};

// TODO: Hablar de lo poderoso que es Derive cuando tenes Senders que implementan Clone :'D
#[derive(Clone)]
pub struct Notifiers {
    sender_log: Sender<LogMessage>,
    command_delegator_sender: Sender<RawCommand>,
}

impl Notifiers {
    pub fn new(
        sender_log: Sender<LogMessage>,
        command_delegator_sender: Sender<RawCommand>,
    ) -> Self {
        Self {
            sender_log,
            command_delegator_sender,
        }
    }
    pub fn write_log(&self, message: LogMessage) {
        let _a = self.sender_log.send(message);
    }

    pub fn off_client(&self, client: &TcpStream, client_fields: Arc<Mutex<ClientFields>>) {
        let (sender_notify, receiver_notify): (Sender<String>, Receiver<String>) = channel();

        let addr = client.peer_addr().unwrap().to_string();
        let command_vec = vec_strings!["clear_client", addr];

        self.command_delegator_sender
            .send((command_vec, sender_notify, client_fields))
            .unwrap();

        match receiver_notify.recv() {
            Ok(_) => {}
            Err(_) => panic!("HELP ME!"),
        }

        self.sender_log
            .send(LogMessage::client_off(&client))
            .unwrap();
    }

    pub fn send_command_delegator(
        &self,
        raw_command: RawCommand,
    ) -> Result<(), SendError<RawCommand>> {
        self.command_delegator_sender.send(raw_command)
    }

    pub fn notify_successful_shipment(
        &self,
        client_fields: Arc<Mutex<ClientFields>>,
        command_received: Vec<String>,
    ) {
        let (sender_notify, receiver_notify): (Sender<String>, Receiver<String>) = channel();

        let mut command_vec_mod = command_received.clone();
        command_vec_mod.insert(0, "notify_monitors".to_string());

        self.command_delegator_sender
            .send((command_vec_mod, sender_notify, Arc::clone(&client_fields)))
            .unwrap();

        match receiver_notify.recv() {
            Ok(_) => {}
            Err(_) => panic!("HELP ME!"),
        }

        self.sender_log
            .send(LogMessage::command_send_by_client(
                &command_received,
                client_fields,
            ))
            .unwrap();
    }
}
