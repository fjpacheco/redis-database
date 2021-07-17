use crate::commands::Runnable;
use crate::communication::log_messages::LogMessage;
use crate::messages::redis_messages;
use crate::native_types::ErrorStruct;
use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

use self::notifiers::Notifiers;

pub mod client_atributes;
pub mod client_handler;
pub mod client_list;
pub mod command_delegator;
pub mod command_subdelegator;
pub mod listener_processor;
pub mod notifiers;
pub mod runnables_map;
pub mod server;

pub type RawCommand = (Vec<String>, Sender<Response>, Arc<Mutex<ClientFields>>);
pub type RawCommandTwo = Option<Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>>;
pub type BoxedCommand<T> = Box<dyn Runnable<T> + Send + Sync>;
pub type Response = Result<String, ErrorStruct>;

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

#[allow(dead_code)]
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

pub fn close_thread(
    thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    name: &str,
    notifer: Notifiers,
) -> Result<(), ErrorStruct> {
    if let Some(handle) = thread {
        handle
            .join()
            .map_err(|_| {
                let _ = notifer.send_log(LogMessage::theard_panic(name)); // I'm not interested ... I retired with the forced Shutdown!
                ErrorStruct::from(redis_messages::thread_panic(name))
            })?
            .and_then(|result| {
                notifer.send_log(LogMessage::theard_closed(name))?;
                Ok(result)
            })
    } else {
        Ok(())
    }
}
