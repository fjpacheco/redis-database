use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::{
    native_types::ErrorStruct,
    redis_config::RedisConfig,
    tcp_protocol::{
        command_delegator::CommandDelegator, database_command_delegator::DatabaseCommandDelegator,
        listener_processor::ListenerProcessor, runnables_map::RunnablesMap,
        server_command_delegator::ServerCommandDelegator,
    },
    Database,
};

use super::command_delegator::CommandsMap;
pub struct ServerRedis {
    // TODO: Change to private and discuss responsibility of methods with Rust-Eze team!
    config: RedisConfig,
    listener_processor: ListenerProcessor,
}

impl ServerRedis {
    pub fn new_port(&mut self, new_port: String) -> Result<String, ErrorStruct> {
        self.listener_processor.new_port(&mut self.config, new_port)
    }

    pub fn start(argv: Vec<String>) -> Result<(), ErrorStruct> {
        // ################## 1° Initialization structures ##################
        let config = RedisConfig::parse_config(argv)?;
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let database = Database::new();

        // ################## 2° Initialization structures ##################
        let (command_delegator_sender, command_delegator_recv) = channel();
        let c_command_delegator_sender = Arc::new(Mutex::new(command_delegator_sender));
        let (commands_map, rcv_cmd_dat, rcv_cmd_sv) = CommandsMap::default();

        // ################## Start listening clients with First Thread ##################
        let listener_processor =
            ListenerProcessor::start(&config, c_command_delegator_sender, clients)?;

        // ################## 3° Initialization structures ##################
        let server_redis = Self::new(config, listener_processor);
        let runnables_database = RunnablesMap::<Database>::database();
        let runnables_server = RunnablesMap::<Self>::server();

        // ################## Start the Three Threads with the important delegators ##################
        CommandDelegator::start(command_delegator_recv, commands_map)?;
        DatabaseCommandDelegator::start(rcv_cmd_dat, runnables_database, database)?;
        join_start_server_command_delegator(rcv_cmd_sv, runnables_server, server_redis)?;

        Ok(())
    }

    fn new(config: RedisConfig, listener_processor: ListenerProcessor) -> Self {
        Self {
            config,
            listener_processor,
        }
    }
}

fn join_start_server_command_delegator(
    rcv_cmd_sv: Receiver<(Vec<String>, Sender<String>)>,
    runnables_server: RunnablesMap<ServerRedis>,
    server_redis: ServerRedis,
) -> Result<(), ErrorStruct> {
    let mut server_command_delegator =
        ServerCommandDelegator::start(rcv_cmd_sv, runnables_server, server_redis)?;
    let join = server_command_delegator.join();
    if let Err(item) = join {
        return Err(ErrorStruct::new(
            "ERR_JOIN_THREAD_SERVER_COMMAND_DELEGATOR".into(),
            format!("{:?}", item),
        ));
    };
    Ok(())
}
