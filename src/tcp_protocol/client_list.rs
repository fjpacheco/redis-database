use crate::tcp_protocol::client_handler::ClientHandler;

pub struct ClientList {
    list: Vec<ClientHandler>,
    log_channel: Sender<LogMessage>
}