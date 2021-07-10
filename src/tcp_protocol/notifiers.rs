use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};

use crate::{communication::log_messages::LogMessage, native_types::ErrorStruct};

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

    pub fn send_log(&self, message: LogMessage) -> Result<(), ErrorStruct> {
        match self.sender_log.send(message) {
            Ok(()) => Ok(()),
            Err(err) => Err(ErrorStruct::new(
                "ERR_SENDER_COMMAND_DELEGATOR".into(),
                err.to_string(),
            )),
        }
    }

    pub fn off_client(&self, addr: String /*, client_fields: Arc<Mutex<ClientFields>>*/) {
        self.sender_log.send(LogMessage::client_off(addr)).unwrap();
    }

    pub fn get_addr(&self, client_fields: Arc<Mutex<ClientFields>>) -> String {
        client_fields.lock().unwrap().get_addr()
    }

    pub fn send_command_delegator(&self, raw_command: RawCommand) -> Result<(), ErrorStruct> {
        match self.command_delegator_sender.send(raw_command) {
            Ok(()) => Ok(()),
            Err(err) => Err(ErrorStruct::new(
                "ERR_SENDER_COMMAND_DELEGATOR".into(),
                err.to_string(),
            )),
        }
    }

    pub fn notify_successful_shipment(
        &self,
        client_fields: Arc<Mutex<ClientFields>>,
        command_received: Vec<String>,
    ) -> Result<(), ErrorStruct> {
        let (sender_notify, receiver_notify) = channel();

        let mut command_vec_modify = command_received.clone();
        command_vec_modify.insert(0, "notify_monitors".to_string());
        let addr = client_fields
            .lock()
            .map(|x| x.get_addr())
            .map_err(|err| ErrorStruct::new("ERR_POISSON_CLIENT_FIELDS".into(), err.to_string()))?;
        command_vec_modify.push(addr);

        self.send_command_delegator((
            command_vec_modify,
            sender_notify,
            Arc::clone(&client_fields),
        ))?;

        match receiver_notify.recv() {
            Ok(_) => {
                self.send_log(LogMessage::command_send_by_client(
                    &command_received,
                    client_fields,
                ))?;
                Ok(())
            }
            Err(err) => Err(ErrorStruct::new(
                "ERR_RECV_ASOCIADO_AL_SENDER_ENVIADO_EN_EL_COMMAND_DELEGATOR".into(),
                err.to_string(),
            )), //TODO: no me maten please
        }
    }
}
