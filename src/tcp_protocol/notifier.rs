use std::{
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
};

use crate::messages::redis_messages;
use crate::{
    communication::log_messages::LogMessage,
    native_types::{error_severity::ErrorSeverity, ErrorStruct},
};

use super::{client_atributes::client_fields::ClientFields, RawCommand};

#[derive(Clone)]
pub struct Notifier {
    sender_log: Sender<Option<LogMessage>>,
    command_delegator_sender: Sender<Option<RawCommand>>,
    status_listener: Arc<AtomicBool>,
    addr_server: String,
}

impl Notifier {
    pub fn new(
        sender_log: Sender<Option<LogMessage>>,
        command_delegator_sender: Sender<Option<RawCommand>>,
        status_listener: Arc<AtomicBool>,
        addr_server: String,
    ) -> Self {
        Self {
            sender_log,
            command_delegator_sender,
            status_listener,
            addr_server,
        }
    }

    /// Si falla, fuerza un cierre de servidor
    pub fn send_log(&self, message: LogMessage) -> Result<(), ErrorStruct> {
        let result_send = self.sender_log.send(Some(message)).map_err(|_| {
            ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::ShutdownServer))
        });

        if let Err(err) = result_send {
            self.force_shutdown_server(err.print_it());
            return Err(err);
        }

        Ok(())
    }

    pub fn off_client(&self, addr: String) -> Result<(), ErrorStruct> {
        self.send_log(LogMessage::client_off(addr))
    }

    pub fn send_command_delegator(
        &self,
        raw_command: Option<RawCommand>,
    ) -> Result<(), ErrorStruct> {
        let result_send = self
            .command_delegator_sender
            .send(raw_command)
            .map_err(|_| {
                ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::ShutdownServer))
            });

        if let Err(err) = result_send {
            self.force_shutdown_server(err.print_it());
            return Err(err);
        }

        Ok(())
    }

    /// Fuerza la desconección del listener del servidor. Comunica a los logs el cierre forzado.
    pub fn force_shutdown_server(&self, reason: String) {
        self.status_listener.store(true, Ordering::SeqCst); // The next connection will necessarily say goodbye.
        let _ = TcpStream::connect(&self.addr_server).map(|_| ()); // TODO: I'm not interested...  🤔
        let _ = self.send_log(LogMessage::forced_shutdown(reason)); // TODO: I'm not interested... x2  🤔
    }

    pub fn notify_successful_shipment(
        &self,
        client_fields: &Arc<Mutex<ClientFields>>,
        command_received: Vec<String>,
    ) -> Result<(), ErrorStruct> {
        let (sender_notify, receiver_notify) = channel();

        let mut command_vec_modify = command_received.clone();
        command_vec_modify.insert(0, "notifymonitors".to_string());
        let addr = client_fields.lock().map(|x| x.get_addr()).map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "client",
                ErrorSeverity::CloseClient,
            ))
        })?;
        command_vec_modify.push(addr);

        self.send_command_delegator(Some((
            command_vec_modify,
            sender_notify,
            Arc::clone(&client_fields),
        )))?;

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
