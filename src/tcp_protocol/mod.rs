use std::sync::mpsc::Sender;

pub mod client_handler;
pub mod command_delegator;
pub mod command_subdelegator;
//pub mod database_command_delegator;
pub mod client_atributes;
pub mod listener_processor;
pub mod runnables_map;
pub mod server;
//pub mod server_command_delegator;

type RawCommand = (Vec<String>, Sender<String>);

/// si te llegó "config get port", el vector te queda "port" y el command_type con "config get"
fn get_command_type(command_input_user: &mut Vec<String>) -> String {
    if command_input_user[0].contains("config") {
        let cmd = "config".to_owned() + " " + &command_input_user[1];
        command_input_user.remove(0);
        command_input_user.remove(0);
        cmd
    } else {
        command_input_user.remove(0)
    }
}
