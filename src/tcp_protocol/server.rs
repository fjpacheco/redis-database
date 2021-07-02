use std::{
    fmt,
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, channel},
        Arc, Mutex,
    },
    time::Duration,
};

use crate::{
    file_manager::FileManager,
    logs::log_center::LogCenter,
    native_types::ErrorStruct,
    redis_config::RedisConfig,
    tcp_protocol::{
        command_delegator::CommandDelegator, /*database_command_delegator::DatabaseCommandDelegator,*/
        listener_processor::ListenerProcessor,
        runnables_map::RunnablesMap,
        /*server_command_delegator::ServerCommandDelegator,*/
    },
    Database,
};

use super::{
    client_list::ClientList, command_delegator::CommandsMap,
    command_subdelegator::CommandSubDelegator, notifiers::Notifiers,
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
        let database = Database::new();

        // ################## 2° Initialization structures ##################
        let (command_delegator_sender, command_delegator_recv) = channel();
        let (commands_map, rcv_cmd_dat, rcv_cmd_sv) = CommandsMap::default();

        // ################## 3° Initialization structures ##################
        let config = Arc::new(Mutex::new(config));
        let status_listener = Arc::new(AtomicBool::new(false));

        // ################## SYSTEM LOG CENTER ##################
        let writer = FileManager::new();
        let (sender_log, receiver) = mpsc::channel();
        let _log_center = LogCenter::new(receiver, Arc::clone(&config), writer);

        // ################## CLIENTS ##################
        let clients = ClientList::new(sender_log.clone());
        let shared_clients = Arc::new(Mutex::new(clients));

        let server_redis = ServerRedisAtributes {
            config: Arc::clone(&config),
            status_listener,
            shared_clients,
        };

        let runnables_database = RunnablesMap::<Database>::database();
        let runnables_server = RunnablesMap::<ServerRedisAtributes>::server();

        // ################## SYSTEM LIST CLIENTS ##################

        // ################## Start the Four Threads with the important delegators and listener ##################
        CommandDelegator::start(command_delegator_recv, commands_map)?;
        CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_database, database)?;
        CommandSubDelegator::start::<ServerRedisAtributes>(
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

        let notifiers = Notifiers::new(sender_log, command_delegator_sender);
        ListenerProcessor::incoming(listener, server_redis, notifiers);
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
