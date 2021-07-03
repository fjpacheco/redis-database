use std::sync::Mutex;
use std::sync::Arc;
use crate::commands::Runnable;
use std::sync::mpsc::Sender;
use crate::tcp_protocol::client_atributes::client_fields::ClientFields;

pub mod client_atributes;
pub mod client_handler;
pub mod client_list;
pub mod command_delegator;
pub mod command_subdelegator;
pub mod listener_processor;
pub mod notifiers;
pub mod runnables_map;
pub mod server;

type RawCommand = (Vec<String>, Sender<String>, Arc<Mutex<ClientFields>>);
type BoxedCommand<T> = Box<dyn Runnable<T> + Send + Sync>;

fn remove_command(command_input_user: &mut Vec<String>) -> String {
    if command_input_user[0].contains("config") & command_input_user.len().eq(&3) {
        let mut cmd = command_input_user.remove(0);
        cmd.push(' ');
        cmd.push_str(&command_input_user.remove(0));
        if cmd.contains("set") {
            cmd.push(' ');
            cmd.push_str(&command_input_user.remove(0));
        }
        cmd
    } else {
        command_input_user.remove(0)
    }
}

fn get_command(command_input_user: &[String]) -> String {
    let mut command_type = command_input_user[0].clone();
    if command_type.contains("config") & command_input_user.len().eq(&3) {
        command_type = command_type.to_owned() + " " + &command_input_user[1].to_string();
        if command_input_user[1].to_string().contains("set") {
            command_type.push(' ');
            command_type.push_str(&command_input_user[2].to_string());
        }
    }
    command_type
}
