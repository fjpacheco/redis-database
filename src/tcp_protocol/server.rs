use std::{
    fmt,
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc, Mutex,
    },
    time::Duration,
};

use crate::{
    file_manager::FileManager,
    joinable::Joinable,
    logs::log_center::LogCenter,
    memory_checker::garbage_collector::GarbageCollector,
    native_types::ErrorStruct,
    redis_config::RedisConfig,
    tcp_protocol::{
        command_delegator::CommandDelegator, listener_processor::ListenerProcessor,
        runnables_map::RunnablesMap,
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
}

#[derive(Clone)]
pub struct ServerRedis;

impl ServerRedis {
    pub fn start(argv: Vec<String>) -> Result<(), ErrorStruct> {
        // ################## 1° Initialization structures: BASIC ELEMENTS ##################
        let config = RedisConfig::parse_config(argv)?;
        let listener = ListenerProcessor::new_tcp_listener(&config)?;
        let database = Database::new();

        // ################## 2° Initialization structures: CHANNELS and COMMANDS MAP ##################
        let (command_delegator_sender, command_delegator_recv) = channel();
        let (sender_log, receiver) = channel();
        let (commands_map, rcv_cmd_dat, rcv_cmd_sv, snd_cmd_dat_garbage) = CommandsMap::default();

        // ################## 3° Initialization structures: SOME PACHMUTEX ##################
        let config = Arc::new(Mutex::new(config));
        let status_listener = Arc::new(AtomicBool::new(false));

        // ################## 4° Initialization structures: CLIENT LIST AND MORE PACHMUTEX ##################
        let clients = ClientList::new(sender_log.clone());
        let shared_clients = Arc::new(Mutex::new(clients));
        let drop_shared_clients = Arc::clone(&shared_clients);

        // ################## 5° Initialization structures: SERVER REDIS ATRIBUTES AND RUNNABLES MAP ##################

        let server_redis = ServerRedisAtributes {
            config: Arc::clone(&config),
            status_listener: status_listener.clone(),
            shared_clients,
        };

        let runnables_database = RunnablesMap::<Database>::database();
        let runnables_server = RunnablesMap::<ServerRedisAtributes>::server();

        // ################## 6° Initialization structures: NOTIFIERS ##################

        let notifiers = Notifiers::new(
            sender_log.clone(),
            command_delegator_sender,
            status_listener,
            server_redis.get_addr(),
        );

        // ################## 7° Initialization structures: STRUCTS WITH THREADS ##################
        let mut log_center = LogCenter::new(
            sender_log,
            receiver,
            Arc::clone(&config),
            FileManager::new(),
        )?;

        let mut command_delegator =
            CommandDelegator::start(command_delegator_recv, commands_map, notifiers.clone())?;
        let mut command_sub_delegator_databse = CommandSubDelegator::start::<Database>(
            rcv_cmd_dat,
            runnables_database,
            database,
            notifiers.clone(),
        )?;
        let mut command_sub_delegator_server_atributes =
            CommandSubDelegator::start::<ServerRedisAtributes>(
                rcv_cmd_sv,
                runnables_server,
                server_redis.clone(),
                notifiers.clone(),
            )?;

        let mut collector = GarbageCollector::start(snd_cmd_dat_garbage, 4, 20, notifiers.clone());

        // ################## ListenerProcessor ##################

        ListenerProcessor::incoming(listener, server_redis, notifiers);

        // ################## FINISH SERVER ##################
        command_delegator.join()?;
        collector.join()?;
        command_sub_delegator_databse.join()?;
        command_sub_delegator_server_atributes.join()?;
        drop_shared_clients.lock().unwrap().join()?;
        log_center.join()?;
        Ok(())
    }
}

impl fmt::Display for ServerRedisAtributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server Redis Atributes")
    }
}
