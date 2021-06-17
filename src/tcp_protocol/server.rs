use std::{
    collections::HashMap,
    fmt,
    sync::{mpsc::channel, Arc, Mutex},
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
    _config: RedisConfig,
}

impl ServerRedis {
    //config set port will not be necessary in the project
    /*pub fn new_port(&mut self, new_port: String) -> Result<String, ErrorStruct> {
        self.listener_processor.new_port(&mut self.config, new_port)
    }*/

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
        let server_redis = Self::new(config);
        let runnables_database = RunnablesMap::<Database>::database();
        let runnables_server = RunnablesMap::<Self>::server();

        // ################## Start the Four Threads with the important delegators and listener ##################
        CommandDelegator::start(command_delegator_recv, commands_map)?;
        CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_database, database)?;
        CommandSubDelegator::start::<ServerRedis>(rcv_cmd_sv, runnables_server, server_redis)?;

        let mut listener_processor =
            ListenerProcessor::start(listener, command_delegator_sender, clients)?;

        let join = listener_processor.join();
        if let Err(item) = join {
            return Err(ErrorStruct::new(
                "ERR_JOIN_LISTENER_PROCESSOR".into(),
                format!("{:?}", item),
            ));
        };
        Ok(())
    }

    fn new(config: RedisConfig) -> Self {
        Self { _config: config }
    }
}

impl fmt::Display for ServerRedis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerRedis")
    }
}
