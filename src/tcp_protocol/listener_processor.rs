use crate::{
    native_types::ErrorStruct, redis_config::RedisConfig,
    tcp_protocol::client_handler::ClientHandler,
};
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener},
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
};

use super::RawCommand;

pub struct ListenerProcessor;

impl ListenerProcessor {
    pub fn incoming(
        listener: TcpListener,
        c_status: Arc<AtomicBool>,
        c_command_delegator_sender: Sender<RawCommand>,
        c_clients: Arc<Mutex<HashMap<SocketAddr, ClientHandler>>>,
    ) {
        for stream in listener.incoming() {
            if c_status.load(std::sync::atomic::Ordering::SeqCst) {
                println!("<Server>: OFF Listener in {:?}", listener);
                break;
            }

            let c_command_delegator_sender = c_command_delegator_sender.clone();
            match stream {
                Ok(client) => {
                    // For debug:
                    let especificacion_cliente: String = "Client: ".to_owned()
                        + "IP: "
                        + client.local_addr().unwrap().to_string().as_str()
                        + " | "
                        + "Peer: "
                        + client.peer_addr().unwrap().to_string().as_str();
                    println!("\n<Server>: Nueva conexión => {}\n", especificacion_cliente);
                    // -----

                    let peer = client.peer_addr().unwrap();
                    //let client_jeje = client.try_clone().unwrap();
                    let new_client = ClientHandler::new(client, c_command_delegator_sender);
                    let mut lock = c_clients.lock().unwrap();
                    // Add user to global hashmap.
                    (*lock).insert(peer, new_client);
                }
                Err(e) => {
                    println!("<Server>: Error to connect client: {:?}", e);
                }
            }
        }
        println!("<Server>: FIN del For de listener.incoming()");
    }

    pub fn new_tcp_listener(config: &RedisConfig) -> Result<TcpListener, ErrorStruct> {
        let ip = config.ip();
        let port = config.port();
        let listener = Self::bind(&ip, &port)?;
        //print!("{}", redis_logo(&port));
        println!("<Server>: Server ON. Bind in: {}", ip + ":" + &port);
        Ok(listener)
    }

    fn bind(ip: &str, port: &str) -> Result<TcpListener, ErrorStruct> {
        match TcpListener::bind(ip.to_owned() + ":" + port) {
            Ok(listener) => Ok(listener),
            Err(error) => Err(ErrorStruct::new(
                "ERR_BIND".into(),
                format!("Bind failure. Detail: {}", error),
            )),
        }
    }
}
