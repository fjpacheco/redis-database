use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};

use crate::messages::redis_messages;
use crate::{
    communication::log_messages::LogMessage,
    native_types::{error_severity::ErrorSeverity, ErrorStruct},
};

use super::{client_atributes::client_fields::ClientFields, RawCommand};

// TODO: Hablar de lo poderoso que es Derive cuando tenes Senders que implementan Clone :'D
#[derive(Clone)]
pub struct Notifier {
    sender_log: Sender<Option<LogMessage>>,
    command_delegator_sender: Sender<Option<RawCommand>>,
}

impl Notifier {
    pub fn new(
        sender_log: Sender<Option<LogMessage>>,
        command_delegator_sender: Sender<Option<RawCommand>>,
    ) -> Self {
        Self {
            sender_log,
            command_delegator_sender,
        }
    }

    pub fn send_log(&self, message: LogMessage) -> Result<(), ErrorStruct> {
        self.sender_log.send(Some(message)).map_err(|_| {
            ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::ShutdownServer))
        })
    }

    pub fn off_client(&self, addr: String /*, client_fields: Arc<Mutex<ClientFields>>*/) {
        let _ = self.sender_log.send(Some(LogMessage::client_off(addr)));
    }

    pub fn send_command_delegator(&self, raw_command: RawCommand) -> Result<(), ErrorStruct> {
        self.command_delegator_sender
            .send(Some(raw_command))
            .map_err(|_| {
                ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::ShutdownServer))
            })
    }

    pub fn notify_successful_shipment(
        &self,
        client_fields: Arc<Mutex<ClientFields>>,
        command_received: Vec<String>,
    ) -> Result<(), ErrorStruct> {
        let (sender_notify, receiver_notify) = channel();

        let mut command_vec_modify = command_received.clone();
        command_vec_modify.insert(0, "notify_monitors".to_string());
        let addr = client_fields.lock().map(|x| x.get_addr()).map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "client",
                ErrorSeverity::CloseClient,
            ))
        })?;
        command_vec_modify.push(addr);

        self.send_command_delegator((
            command_vec_modify,
            sender_notify,
            Arc::clone(&client_fields),
        ))?;

        receiver_notify
            .recv()
            .map(|_| ())
            .map_err(|_| {
                ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::Comunicate))
            })
            .and_then(|_| {
                self.send_log(LogMessage::command_send_by_client(
                    &command_received,
                    client_fields,
                ))
            })
    }
}
