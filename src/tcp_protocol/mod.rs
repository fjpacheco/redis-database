use std::sync::mpsc::Sender;

pub mod client_handler;
pub mod command_delegator;
pub mod command_subdelegator;
//pub mod database_command_delegator;
pub mod listener_processor;
pub mod runnables_map;
pub mod server;
//pub mod server_command_delegator;


type RawCommand = (Vec<String>, Sender<String>);
