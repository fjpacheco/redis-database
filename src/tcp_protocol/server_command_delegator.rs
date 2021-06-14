use std::any::Any;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::native_types::ErrorStruct;
use crate::native_types::RError;
use crate::native_types::RedisType;

use crate::tcp_protocol::runnables_map::RunnablesMap;
use crate::tcp_protocol::server::ServerRedis;

pub struct ServerCommandDelegator {
    database_command_handler: Option<JoinHandle<()>>,
}

impl ServerCommandDelegator {
    pub fn start(
        rcv_sv: Receiver<(Vec<String>, Sender<String>)>,
        runnables_server: RunnablesMap<ServerRedis>,
        mut server_redis: ServerRedis,
    ) -> Result<Self, ErrorStruct> {
        let builder = thread::Builder::new().name("Server Command Delegator".into());

        let database_command_handler = builder.spawn(move || {
            for (mut command_input_user, sender_to_client) in rcv_sv.iter() {
                let command_type = get_command_type(&mut command_input_user);
                if let Some(runnable_command) = runnables_server.get(&command_type) {
                    // REMEMBER TO CHANGE THE TRAIT (MUST WORK WITH VEC<STRING>)
                    let command_str: Vec<&str> =
                        command_input_user.iter().map(|s| s.as_ref()).collect();
                    match runnable_command.run(command_str, &mut server_redis) {
                        Ok(encoded_resp) => sender_to_client.send(encoded_resp).unwrap(),
                        Err(err) => sender_to_client.send(RError::encode(err)).unwrap(),
                    };
                } else {
                    let error =
                        ErrorStruct::new("ERR".to_string(), "command does not exist".to_string());
                    sender_to_client.send(RError::encode(error)).unwrap();
                }
            }
        });

        match database_command_handler {
            Ok(item) => Ok(Self {
                database_command_handler: Some(item),
            }),
            Err(item) => Err(ErrorStruct::new("ERR_THREAD".into(), format!("{}", item))),
        }
    }

    pub fn join(&mut self) -> Result<(), Box<dyn Any + Send>> {
        self.database_command_handler.take().unwrap().join()
    }
}

// TODO: refactorizar !
/// si te llegó "config set port 9000", el vector te queda "port 9000" y el command_type con "config set"
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
