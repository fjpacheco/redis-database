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

/// Structure in charge of sending the communication [RawCommand] and [LogMessage] to the threads of
/// the main structures of the [CommandDelegator](crate::tcp_protocol::command_delegator::CommandDelegator) and [LogCenter](crate::logs::log_center::LogCenter) respectively.
///
/// It also has the ability to force a server shutdown.
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

    /// A [LogMessage] will be sent to the [LogCenter](crate::logs::log_center::LogCenter) for processing.
    /// If the communication channel is closed, the server is forced to close.
    ///
    /// # Error
    /// Returns an [ErrorStruct] if:
    ///
    /// * The channel to communicate with the [LogCenter](crate::logs::log_center::LogCenter) is closed.
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

    /// The [LogCenter](crate::logs::log_center::LogCenter) will be notified of the disconnection of a client from the server.
    ///
    /// # Error
    /// Returns an [ErrorStruct] if:
    ///
    /// * The channel to communicate with the [LogCenter](crate::logs::log_center::LogCenter) is closed.
    pub fn off_client(&self, addr: String) -> Result<(), ErrorStruct> {
        self.send_log(LogMessage::client_off(addr))
    }

    /// The [RawCommand] package will be sent to the CommandDelegator for processing.
    /// If the communication channel is closed, the server is forced to shut down.     
    ///
    /// # Error
    /// Returns an [ErrorStruct] if:
    ///
    /// * The channel to communicate with the [CommandDelegator](crate::tcp_protocol::command_delegator::CommandDelegator) is closed.
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

    /// Force disconnection of [ListenerProcessor](crate::tcp_protocol::listener_processor::ListenerProcessor). It informs the logs of the forced closure.
    pub fn force_shutdown_server(&self, reason: String) {
        self.status_listener.store(true, Ordering::SeqCst); // The next connection will necessarily say goodbye.
        let _ = TcpStream::connect(&self.addr_server).map(|_| ());
        let _ = self.send_log(LogMessage::forced_shutdown(reason));
    }

    /// Each client with [Status::Monitor](crate::tcp_protocol::client_atributes::status::Status) from [ClientList](crate::tcp_protocol::client_list::ClientList) receives a notification of all commands processed successfully on the server.
    /// This is done by sending a special command through the [CommandDelegator](crate::tcp_protocol::command_delegator::CommandDelegator)
    ///
    /// # Error
    /// Returns an [ErrorStruct] if:
    ///
    /// * The channel to communicate with the [CommandDelegator](crate::tcp_protocol::command_delegator::CommandDelegator) or the [LogCenter](crate::logs::log_center::LogCenter) is closed.
    /// * Received client fields have been poisoned.
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
