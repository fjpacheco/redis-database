use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::native_types::ErrorStruct;
use crate::native_types::RError;
use crate::native_types::RedisType;

use crate::tcp_protocol::runnables_map::RunnablesMap;
use crate::tcp_protocol::server::ServerRedis;

pub struct ServerCommandDelegator;

impl ServerCommandDelegator {
    pub fn start(
        rcv_cmd_server: Receiver<(Vec<String>, Sender<String>)>,
        runnables_map: RunnablesMap<ServerRedis>,
        mut server_redis: ServerRedis,
    ) -> Result<(), ErrorStruct> {
        let _database_command_handler = thread::spawn(move || {
            for (mut command_input_user, sender_to_client) in rcv_cmd_server.iter() {
                let command_type = get_command_type(&mut command_input_user);
                if let Some(runnable_command) = runnables_map.get(&command_type) {
                    // REMEMBER TO CHANGE THE TRAIT (MUST WORK WITH VEC<STRING>)
                    let command_str: Vec<&str> =
                        command_input_user.iter().map(|s| s.as_ref()).collect();
                    match runnable_command.run(command_str, &mut server_redis) {
                        // TODO: despues de refactorizar el run() => acá recibe piolamente el serverRedis
                        Ok(encoded_resp) => sender_to_client.send(encoded_resp).unwrap(),
                        Err(err) => sender_to_client.send(RError::encode(err)).unwrap(),
                    };
                } else {
                    let error =
                        ErrorStruct::new("ERR".to_string(), "command does not exist".to_string());
                    sender_to_client.send(RError::encode(error)).unwrap();
                }

                /*let c_listener_procesor = listener_procesor.clone();

                // Esto esta SUPER-HARDCODEADO! Deberia existir un config_set.rs y laburar ahí ésto!!!!!!
                // Solo tomenlo como idea!
                // Acá SOLAMENTE TRABAJO CON EL COMANDO "CONFIG SET PORT" .. Ejemplo "config set port 9000"
                // Te cambiará todo al port 9000, y se muere el anterior port con el listener viejo!!
                // TODO ESTO se hace gracias al metodo de ListenerProcesosr::new_port()
                let respuesta = (*c_listener_procesor)
                    .lock()
                    .unwrap()
                    .new_port(config.clone(), command_input_user);


                sender_to_client.send("+OKA\r\n".to_string());*/
            }
        })
        .join();
        Ok(())
    }
}

// TODO: refactorizar !
/// si te llegó "config set port 9000", el vector te queda "port 9000" y el command_type con "config set"
fn get_command_type(command_input_user: &mut Vec<String>) -> String {
    if command_input_user[0].contains("config") {
        let cmd = "config".to_owned() + &command_input_user[1];
        command_input_user.remove(0);
        command_input_user.remove(1);
        cmd
    } else {
        command_input_user.remove(0)
    }
}
