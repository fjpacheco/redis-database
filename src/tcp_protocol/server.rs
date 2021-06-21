use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc, Mutex,
    },
};

use crate::{
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

use super::{command_delegator::CommandsMap, command_subdelegator::CommandSubDelegator};
pub struct ServerRedis {
    config: RedisConfig,
    status: Arc<AtomicBool>,
}

impl ServerRedis {
    pub fn store(&self, val: bool) {
        self.status.store(val, Ordering::SeqCst);
    }
    pub fn get_addr(&self) -> String {
        self.config.get_addr()
    }

    pub fn start(argv: Vec<String>) -> Result<(), ErrorStruct> {
        // ################## 1° Initialization structures ##################
        let config = RedisConfig::parse_config(argv)?;
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let database = Database::new();
        let listener = ListenerProcessor::new_tcp_listener(&config)?;

        // ################## 2° Initialization structures ##################
        let (command_delegator_sender, command_delegator_recv) = channel();
        let (commands_map, rcv_cmd_dat, rcv_cmd_sv) = CommandsMap::default();

        // ################## 3° Initialization structures ##################
        let status = Arc::new(AtomicBool::new(false));
        let server_redis = Self {
            config,
            status: status.clone(),
        };
        let runnables_database = RunnablesMap::<Database>::database();
        let runnables_server = RunnablesMap::<Self>::server();

        // ################## Start the Four Threads with the important delegators and listener ##################
        CommandDelegator::start(command_delegator_recv, commands_map)?;
        CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_database, database)?;
        CommandSubDelegator::start::<ServerRedis>(rcv_cmd_sv, runnables_server, server_redis)?;

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

        ListenerProcessor::incoming(listener, status, command_delegator_sender, clients);
        Ok(())
    }
}

impl fmt::Display for ServerRedis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerRedis")
    }
}

impl Drop for ServerRedis {
    fn drop(&mut self) {
        println!("Dropping ServerRedis 😜");
    }
}
