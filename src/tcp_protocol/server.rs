use std::{
    fmt,
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, channel, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

use crate::{
    communication::log_messages::LogMessage,
    file_manager::FileManager,
    logs::log_center::LogCenter,
    native_types::ErrorStruct,
    redis_config::RedisConfig,
    tcp_protocol::{
        client_atributes::client_fields::ClientFields, command_delegator::CommandDelegator,
        listener_processor::ListenerProcessor, notifier::Notifier, runnables_map::RunnablesMap,
    },
    vec_strings, Database,
};

use super::{
    client_list::ClientList, command_delegator::CommandsMap,
    command_subdelegator::CommandSubDelegator,
};
#[derive(Clone)]
pub struct ServerRedisAtributes {
    config: Arc<Mutex<RedisConfig>>,
    status_listener: Arc<AtomicBool>,
    pub shared_clients: Arc<Mutex<ClientList>>,
}

impl ServerRedisAtributes {
    pub fn get_timeout(&self) -> String {
        self.config
            .lock()
            .expect("ERROR IN REDIS CONFIG POISSONED")
            .timeout()
            .to_string()
    }

    pub fn get_client_list(&self) -> Arc<Mutex<ClientList>> {
        Arc::clone(&self.shared_clients)
    }

    pub fn store(&self, val: bool) {
        self.status_listener.store(val, Ordering::SeqCst);
    }

    pub fn set_timeout(&self, client: &TcpStream) {
        let time = self
            .config
            .lock()
            .expect("ERROR IN REDIS CONFIG POISSONED")
            .timeout();
        if time.gt(&0) {
            client
                .set_read_timeout(Some(Duration::new(time, 0)))
                .expect("ERROR FOR SET TIMEOUT IN CLIENT");
        }
    }

    pub fn get_addr(&self) -> String {
        self.config
            .lock()
            .expect("ERROR IN REDIS CONFIG POISSONED")
            .get_addr()
    }

    pub fn get_port(&self) -> String {
        self.config
            .lock()
            .expect("ERROR IN REDIS CONFIG POISSONED")
            .port()
    }

    pub fn is_listener_off(&self) -> bool {
        self.status_listener
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /*pub fn insert_client(&mut self, new_client: ClientHandler) {
        self.clients.insert(new_client);
    }*/
}

#[derive(Clone)]
pub struct ServerRedis;

impl ServerRedis {
    pub fn start(argv: Vec<String>) -> Result<(), ErrorStruct> {
        // ################## 1° Initialization structures ##################
        let config = RedisConfig::parse_config(argv)?;
        let listener = ListenerProcessor::new_tcp_listener(&config)?;

        // ################## 2° Initialization structures ##################
        let (command_delegator_sender, command_delegator_recv) = channel();
        let (commands_map, rcv_cmd_dat, rcv_cmd_sv) = CommandsMap::default();

        // ################## 3° Initialization structures ##################
        let config = Arc::new(Mutex::new(config));
        let status_listener = Arc::new(AtomicBool::new(false));

        // ################## SYSTEM LOG CENTER ##################
        let writer = FileManager::new();
        let (sender_log, receiver) = mpsc::channel();
        let _log_center =
            LogCenter::new(sender_log.clone(), receiver, Arc::clone(&config), writer)?;

        // ################## CLIENTS ##################
        let clients = ClientList::new(sender_log.clone());
        let shared_clients = Arc::new(Mutex::new(clients));
        let drop_shared_clients = Arc::clone(&shared_clients);

        let server_redis = ServerRedisAtributes {
            config: Arc::clone(&config),
            status_listener,
            shared_clients,
        };

        let notifier = Notifier::new(sender_log.clone(), command_delegator_sender.clone());
        let database = Database::new(notifier.clone());
        let runnables_database = RunnablesMap::<Database>::database();
        let runnables_server = RunnablesMap::<ServerRedisAtributes>::server();

        // ################## SYSTEM LIST CLIENTS ##################

        // ################## Start the Four Threads with the important delegators and listener ##################
        let c_commands_map = Arc::new(Mutex::new(commands_map));

        let a = CommandDelegator::start(
            command_delegator_sender.clone(),
            command_delegator_recv,
            Arc::clone(&c_commands_map),
        )?;
        let b = CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_database, database)?;
        let c = CommandSubDelegator::start::<ServerRedisAtributes>(
            rcv_cmd_sv,
            runnables_server,
            server_redis.clone(),
        )?;

        /*
        ¡CRATE EXTERNO!
        CONTROLAS EL SIGINT CRTL+C


        [dependencies]
        ctrlc = { version = "3.0", features = ["termination"] }


        ctrlc::set_handler(move ||  {
            let client = redis::Client::open(
                "redis://".to_owned()
                + &c_config.ip()
                + ":"
                + &c_config.port()
                + "/",
            )
            .unwrap();
            let mut conection_client = client.get_connection().unwrap();

            let _received_3: Result<String, RedisError> = redis::cmd("shutdown")
                .query(&mut conection_client);

        })
        .expect("Error setting Ctrl-C handler");
        */

        // TODO: EN CONSTRUCCIÓN.. ESTO ESTA MUY FEO! Hay que tener en cuenta el análisis de unwraps!!!!!!!!!!!!
        ListenerProcessor::incoming(listener, server_redis.clone(), notifier.clone());

        let (sender_drop, recv_drop) = channel();
        command_delegator_sender
            .send(Some((
                vec_strings!["OK"],
                sender_drop,
                Arc::new(Mutex::new(ClientFields::new(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    8080,
                )))),
            )))
            .unwrap(); // NECESITO SI O SI LA STRUC PACKAGE_FOR_SEND
        println!("arranco a esperar a estos giles qls");
        std::thread::spawn(move || match recv_drop.recv() {
            Ok(_) => {
                drop(drop_shared_clients);

                c_commands_map.lock().unwrap().kill_senders();

                drop(command_delegator_sender);
                drop(sender_log);
                drop(notifier);
                drop(server_redis);
                drop(a);
                drop(b);
                drop(c);
            }
            Err(_) => todo!("Help me! Se cerró el server con errores..."),
        })
        .join()
        .unwrap();

        /*drop(drop_shared_clients);
        c_commands_map.lock().unwrap().kill_senders();
        drop(command_delegator_sender);
        drop(sender_log);
        drop(notifier);
        drop(server_redis);
        drop(a);
        drop(b);
        drop(c);*/
        Ok(())
    }
}

impl fmt::Display for ServerRedisAtributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server Redis Atributes")
    }
}

impl Drop for ServerRedis {
    fn drop(&mut self) {
        println!("Dropping ServerRedis 😜");
    }
}
