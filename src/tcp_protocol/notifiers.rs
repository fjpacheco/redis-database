use std::sync::{
    mpsc::{channel, Receiver, SendError, Sender},
    Arc, Mutex,
};

use crate::communication::log_messages::LogMessage;

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

    pub fn off_client(&self, addr: String /*, client_fields: Arc<Mutex<ClientFields>>*/) {
        self.sender_log
            .send(LogMessage::client_off(addr.to_string()))
            .unwrap();

        /*let (sender_notify, receiver_notify): (Sender<String>, Receiver<String>) = channel();
        let command_vec = vec_strings!["clear_client", &addr];

        self.command_delegator_sender
            .send((command_vec, sender_notify, client_fields))
            .unwrap();

        match receiver_notify.recv() {
            Ok(_) => {}
            Err(_) => panic!("HELP ME!"),
        }*/
    }

    pub fn get_addr(&self, client_fields: Arc<Mutex<ClientFields>>) -> String {
        client_fields.lock().unwrap().get_addr()
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
        command_vec_mod.push(client_fields.lock().unwrap().get_addr());
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
