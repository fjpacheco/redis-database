use crate::native_types::ErrorStruct;
use std::{
    error::Error,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::tcp_protocol::client_atributes::client_fields::ClientFields;

/// This structure contains the message that
/// should be dumped into the log file, and
/// a number that represents the verbose
/// priority, that indicates if the message
/// should be printed through standar output.
pub struct LogMessage {
    verbose_priority: usize,
    message: Option<String>,
}

impl LogMessage {
    /// Creates the log message.
    pub fn new(verbose_priority: usize, message: String) -> LogMessage {
        LogMessage {
            verbose_priority,
            message: Some(message),
        }
    }

    /// Creates the log message from an ErrorStruct.
    pub fn from_errorstruct(error: ErrorStruct) -> LogMessage {
        LogMessage {
            verbose_priority: 1,
            message: Some(error.print_it()),
        }
    }

    /// Returns the message if the verbose value match.
    pub fn is_verbosely_printable(&self, verbose: &usize) -> Option<&String> {
        if *verbose > self.verbose_priority {
            self.message.as_ref()
        } else {
            None
        }
    }

    // Removes and return the message.
    pub fn take_message(&mut self) -> Option<String> {
        self.message.take()
    }

    pub fn test_message1() -> LogMessage {
        LogMessage::new(2, "This is test 1".to_string())
    }

    pub fn test_message2() -> LogMessage {
        LogMessage::new(2, "This is test 2".to_string())
    }

    pub fn test_message3() -> LogMessage {
        LogMessage::new(2, "This is test 3".to_string())
    }

    // A partir de aca abajo, pondriamos todos los
    // mensajes que queremos logear.

    /// COMMAND --> KEY: VALUE
    pub fn database_correctly_updated(formatted_data: String) -> LogMessage {
        LogMessage::new(9, format!("Database update: {}", formatted_data))
    }

    pub fn off_server(listener: &TcpListener) -> LogMessage {
        LogMessage::new(
            2,
            format!("Server OFF in {:?}", listener.local_addr().unwrap()),
        )
    }

    pub fn start_up(listener: &TcpListener) -> LogMessage {
        LogMessage::new(
            2,
            format!("Server ON in {:?}", listener.local_addr().unwrap()),
        )
    }

    pub fn error_to_connect_client(err: &dyn Error) -> LogMessage {
        LogMessage::new(2, format!("Error to connect client: {:?}", err))
    }

    pub fn log_stopped() -> LogMessage {
        LogMessage::new(2, "Log center has stopped working.".to_string())
    }

    pub fn forced_shutdown(reason: String) -> LogMessage {
        LogMessage::new(
            2,
            format!("Server is shutdown in a forced way. DETAIL: {}", reason),
        )
    }

    pub fn log_closed_success() -> LogMessage {
        LogMessage::new(2, "Log center is closed.".to_string())
    }

    pub fn command_send_by_client(
        command: &[String],
        client_fields: &Arc<Mutex<ClientFields>>,
    ) -> LogMessage {
        let addr = client_fields.lock().unwrap().get_addr();
        LogMessage::new(2, format!("[{}] {:?}", addr, command))
    }

    pub fn client_off(client_addr: String) -> LogMessage {
        LogMessage::new(2, format!("Client disconected: {:?}", client_addr))
    }

    pub fn new_conection(client: &TcpStream) -> LogMessage {
        LogMessage::new(
            2,
            format!("New conection: {:?}", client.peer_addr().unwrap()),
        )
    }

    pub fn theard_panic(name_thread: &str) -> LogMessage {
        LogMessage::new(2, format!("Thread '{}' has panicked", name_thread))
    }

    pub fn channel_client_off() -> LogMessage {
        LogMessage::new(
            2,
            "Error to send message a client. The channel of client is disconected.".to_string(),
        )
    }

    pub fn theard_closed(name_thread: &str) -> LogMessage {
        LogMessage::new(2, format!("Thread '{}' is closed.", name_thread))
    }

    pub fn key_touched(key: &str, time: u64) -> LogMessage {
        LogMessage::new(
            2,
            format!("Key {} has been updated. Last access: {}", key, time),
        )
    }

    pub fn detail_clients(clients_detail: Vec<String>) -> LogMessage {
        let mut message = String::from("List of clients:\r\n");
        for (index, client) in clients_detail.iter().enumerate() {
            message.push_str("               -> ");
            message.push_str(&client);
            if !(clients_detail.len() - 1).eq(&(index)) {
                message.push_str("\r\n")
            }
        }
        LogMessage::new(2, message)
    }
}

#[cfg(test)]
pub mod test_log_messages {

    use super::*;

    #[test]
    fn test01_print_a_log_message() {
        let message = LogMessage::new(3, "This is a test".to_string());
        let verbose = 4;
        assert_eq!(
            message.is_verbosely_printable(&verbose),
            Some(&"This is a test".to_string())
        );
    }

    #[test]
    fn test02_can_not_print_because_of_greater_verbose() {
        let message = LogMessage::new(3, "This is a test".to_string());
        let verbose = 2;
        assert_eq!(message.is_verbosely_printable(&verbose), None);
    }

    #[test]
    fn test03_get_the_message_no_matter_which_verbose() {
        let mut message = LogMessage::new(3, "This is a test".to_string());
        assert_eq!(message.take_message(), Some("This is a test".to_string()));
        assert_eq!(message.take_message(), None);
    }
}
