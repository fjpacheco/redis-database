use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
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
    // TODO: crear metodos para get_mut().. No hacer publico los args
    pub config: Arc<Mutex<RedisConfig>>,
    pub listener_processor: Option<Arc<Mutex<ListenerProcessor>>>,
    // pub runnables_map: RunnablesMap, // TODO: revisar esto!
}

impl ServerRedis {
    // No me interesa devolver un ServerRedis! Por eso deuvelvo Ok(()) que nunca se usará porque se queda trabado el flujo acá dentro del ::start() (leer más adelante lo aclaro mejor!)
    // Si quisiese comunicarme con el ServerRedis, o ejecutar funcion sobre él => SOLAMENTE el CLiente lo deberia hacer!
    // Si un cliente ejecuta cambios al ServerRedis => Entonces dentro del ServerRedis deberá esperar, con un channel, con commandos a ejecutar!!! O eso estuve pensando!!!
    pub fn start(argv: Vec<String>) -> Result<(), ErrorStruct> {
        // Configuración => No darle mucha bola a esto!
        let config = match argv.len().eq(&2) {
            true => RedisConfig::get_with_new_config(&argv[1])?,
            false => RedisConfig::default(),
        };

        let config = Arc::new(Mutex::new(config));

        let config_copy = match argv.len().eq(&2) {
            true => RedisConfig::get_with_new_config(&argv[1])?,
            false => RedisConfig::default(),
        };

        // Los demás atributos del server que deberian ir a una sub-estructura!
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let database = Database::new();

        // Se crean delegadores y sus RunnableMaps/CommandsMap!
        // Ademas recibo el nuevo 'rcv_sv':
        let (command_delegator_sender, command_delegator_recv): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = std::sync::mpsc::channel();

        let (commands_map, rcv_cmd_dat, rcv_sv) = CommandsMap::default();

        let mut server_redis = ServerRedis {
            config,
            listener_processor: None,
        };

        let _database_command_delegator = DatabaseCommandDelegator::new(
            rcv_cmd_dat,
            RunnablesMap::<Database>::database(),
            database,
        ); // TODO: pasar el ServerRedis!! Cambiar los runs para recibir ServerRedis !!
        let _command_delegator = CommandDelegator::new(command_delegator_recv, commands_map);
        let command_delegator_sender = Arc::new(Mutex::new(command_delegator_sender));

        // Construyo el ListenerProcessor ....tambien podrian ir a una sub-estructura!
        // En ésta estructura me pongo a escuchar a los clientes dentro de ese ListenerProcessor::start()
        // Se podria evitar ese metodo "new_tcp_listener". Es redundante creo
        // let listener = ListenerProcessor::new_tcp_listener(&config.lock().unwrap())?; // No hace falta Arc acá..... el primer Lock usado no hace falta matchear el Result => Never Poison!
        let listener = ListenerProcessor::new_tcp_listener(&config_copy)?; // No hace falta Arc acá..... el primer Lock usado no hace falta matchear el Result => Never Poison!
        let listener_processor = Arc::new(Mutex::new(ListenerProcessor::start(
            listener,
            command_delegator_sender.clone(),
            clients.clone(),
        )));

        server_redis.listener_processor = Some(listener_processor); // TODO: ver si podes evitar el Option acá!!!!

        ServerCommandDelegator::start(rcv_sv, RunnablesMap::<ServerRedis>::server(), server_redis)?;

        // Nunca deberia llegar el flujo de programa acá. TOTALMENTE IRRELEVANTE, pero habia que poner el Ok por el compilador xd
        Ok(())
    }
}
