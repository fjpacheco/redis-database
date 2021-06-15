use std::sync::mpsc::Sender;

pub mod client_handler;
pub mod command_delegator;
pub mod database_command_delegator;
pub mod listener_processor;
pub mod runnables_map;
pub mod server;
pub mod server_command_delegator;

pub enum SendersRedis {
    VectorAndSender((Vec<String>, Sender<String>)),
}
