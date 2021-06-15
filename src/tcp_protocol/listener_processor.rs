use crate::{
    native_types::ErrorStruct, redis_config::RedisConfig,
    tcp_protocol::client_handler::ClientHandler,
};
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct ListenerProcessor {
    thread_processor: JoinHandle<()>,
    command_delegator_sender: Arc<Mutex<Sender<(Vec<String>, Sender<String>)>>>,
    clients: Arc<Mutex<HashMap<SocketAddr, ClientHandler>>>,
    status: Arc<AtomicBool>,
}

impl ListenerProcessor {
    pub fn new_port(
        &mut self,
        config: &mut RedisConfig,
        new_port: String,
    ) -> Result<String, ErrorStruct> {
        let ip_old = config.ip();
        let port_old = config.port();

        config.update_port(&new_port);
        let listener_new = match Self::new_tcp_listener(&config) {
            Ok(item) => item,
            Err(err) => {
                config.update_port(&port_old);
                return Err(ErrorStruct::new(
                    "ERR_UPDATE".to_string(),
                    format!("Error to update port with new tcp listener {:?}", err),
                ));
            }
        };

        // Lo pongo en true a ese "status" cosa de que el while se de cuenta en la proxima vuelta!!!
        self.status.store(true, Ordering::SeqCst);

        // Y al instante de updatear a true => hago una conexion fantasma con un cliente!
        //  con esto te aprovechas el break dentro del for in listener.incoming().iter() del listener viejo!!!! Con eso CORTÁS ESE BUCLE!!
        match TcpStream::connect(ip_old + ":" + &port_old) {
            Ok(_) => Ok(self.updated(listener_new)),
            Err(err) => {
                // Vuelvo a la normalidad el status, no se logró conectar ese cliente fantasma
                self.status.store(false, Ordering::SeqCst);
                config.update_port(&port_old);
                return Err(ErrorStruct::new(
                    "ERR_CONNECT".to_string(),
                    format!("Error to Connect False Client: {:?}", err),
                ));
            }
        }
    }

    fn updated(&mut self, listener: TcpListener) -> String {
        let cc_command_delegator_sender = self.command_delegator_sender.clone();
        let cc_clients = self.clients.clone();

        let cc_status = self.status.clone();

        let thread_processor = thread::spawn(move || {
            Self::incoming(listener, cc_status, cc_command_delegator_sender, cc_clients);
        });

        self.thread_processor = thread_processor;
        String::from("+NEW PORT! <- NO TE OLVIDES MODIFICAR ÉSTE MENSAJE\r\n")
    }

    pub fn start(
        config: &RedisConfig,
        command_delegator_sender: Arc<Mutex<Sender<(Vec<String>, Sender<String>)>>>,
        clients: Arc<Mutex<HashMap<SocketAddr, ClientHandler>>>,
    ) -> Result<Self, ErrorStruct> {
        let cc_command_delegator_sender = command_delegator_sender.clone();
        let cc_clients = clients.clone();

        let status = Arc::new(AtomicBool::new(false));
        let cc_status = Arc::clone(&status);
        let listener = Self::new_tcp_listener(config)?;

        let thread_processor = thread::spawn(move || {
            Self::incoming(listener, cc_status, cc_command_delegator_sender, cc_clients);
        });

        let listener = ListenerProcessor {
            thread_processor,
            command_delegator_sender,
            clients,
            status,
        };
        Ok(listener)
    }

    fn incoming(
        listener: TcpListener,
        status: Arc<AtomicBool>,
        cc_command_delegator_sender: Arc<Mutex<Sender<(Vec<String>, Sender<String>)>>>,
        cc_clients: Arc<Mutex<HashMap<SocketAddr, ClientHandler>>>,
    ) {
        for stream in listener.incoming() {
            if status.load(std::sync::atomic::Ordering::SeqCst) {
                status.store(false, Ordering::SeqCst);
                println!("<Server>: OFF Listener in {:?}", listener);
                break;
            }

            let c_command_delegator_sender = cc_command_delegator_sender.lock().unwrap().clone(); // Cierro RAPIDAMENTE el lock.. desbloqueo digamos cosa de que OTRO lo peuda usar
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
                    let new_client = ClientHandler::new(client, c_command_delegator_sender);
                    let mut lock = cc_clients.lock().unwrap();
                    // Add user to global hashmap.
                    (*lock).insert(peer, new_client);
                    println!("<Server>: Clientes: \n {:?} \n", *lock);
                }
                Err(e) => {
                    println!("<Server>: Error to connect client: {:?}", e);
                }
            }
            println!("<Server>: Mientras tanto sigo esperando una nueva conexión... \n");
        }
    }

    pub fn new_tcp_listener(config: &RedisConfig) -> Result<TcpListener, ErrorStruct> {
        let ip = config.ip();
        let port = config.port();
        let listener = bind(&ip, &port)?;
        //print!("{}", redis_logo(&port));
        println!("<Server>: Server ON. Bind in: {}", ip + ":" + &port);
        Ok(listener)
    }
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
