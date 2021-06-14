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
        config: Arc<Mutex<RedisConfig>>,
        command: Vec<String>,
    ) -> Result<String, ErrorStruct> {
        let mut config = match config.lock() {
            Ok(item) => item,
            Err(err) => {
                return Err(ErrorStruct::new(
                    "ERR_PORT".to_string(),
                    format!("Error to access ConfigRedis: {:?}", err),
                ))
            }
        };
        let ip_old = config.ip().to_string();
        let port_old = config.port().to_string();

        let new_port = match command.get(3) {
            // es el tercero del input! super hardcodeado
            Some(item) => item,
            None => {
                return Err(ErrorStruct::new(
                    "ERR_PORT".to_string(),
                    "Port not found in input".to_string(),
                ))
            }
        };

        config.update_port(new_port);
        let listener_new = match Self::new_tcp_listener(&config) {
            Ok(item) => item,
            Err(err) => {
                return Err(ErrorStruct::new(
                    "ERR_UPDATE".to_string(),
                    format!("Error to update port with new tcp listener {:?}", err),
                ))
            }
        };

        // Lo pongo en true a ese "StatusListener" cosa de que el while se de cuenta en la proxima vuelta!!!
        // En vez de una estructura, me creo
        self.status.store(true, Ordering::SeqCst);

        // Y al instante de updatear a true => hago una conexion fantasma con un cliente!
        //  con esto te aprovechas el break dentro del for in listener.incoming().iter() del listener viejo!!!! Con eso CORTÁS ESE BUCLE!!
        match TcpStream::connect(ip_old.to_owned() + ":" + &port_old) {
            Ok(_) => Ok(self.updated(listener_new)),
            Err(err) => {
                self.status.store(false, Ordering::SeqCst);
                //self.status_update_to_false()?; // Vuelvo a la normalidad el status, no se logró conectar ese cliente fantasma :C
                config.update_port(&port_old); // Dejo todo como estaba antes!
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
        self.status.store(false, Ordering::SeqCst);

        let cc_status = self.status.clone();

        let thread_processor = thread::spawn(move || {
            Self::incoming(listener, cc_status, cc_command_delegator_sender, cc_clients);
        });

        self.thread_processor = thread_processor;
        String::from("+TODO PIOLA PA\r\n") // Esto esta mal
    }

    pub fn start(
        listener: TcpListener,
        command_delegator_sender: Arc<Mutex<Sender<(Vec<String>, Sender<String>)>>>,
        clients: Arc<Mutex<HashMap<SocketAddr, ClientHandler>>>,
    ) -> Self {
        let cc_command_delegator_sender = command_delegator_sender.clone();
        let cc_clients = clients.clone();

        let status = Arc::new(AtomicBool::new(false));
        let cc_status = Arc::clone(&status);

        let thread_processor = thread::spawn(move || {
            Self::incoming(listener, cc_status, cc_command_delegator_sender, cc_clients);
        });

        ListenerProcessor {
            thread_processor,
            clients,
            command_delegator_sender,
            status,
        }
    }

    fn incoming(
        listener: TcpListener,
        status: Arc<AtomicBool>,
        cc_command_delegator_sender: Arc<Mutex<Sender<(Vec<String>, Sender<String>)>>>,
        cc_clients: Arc<Mutex<HashMap<SocketAddr, ClientHandler>>>,
    ) {
        for stream in listener.incoming() {
            /*if status.lock().unwrap().new_listener() {
                (*status).lock().unwrap().update_to_false();
                println!("<Server>: OFF Listener in {:?}", listener);
                break;
            }*/

            if status.load(std::sync::atomic::Ordering::SeqCst) {
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
        println!(
            "<Server>: Server ON. Bind in: {}",
            ip.to_owned() + ":" + &port
        );
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
