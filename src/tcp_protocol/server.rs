use std::sync::{atomic::AtomicBool, mpsc::channel, Arc, Mutex};

use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;
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
    command_subdelegator::CommandSubDelegator, notifier::Notifier,
};

#[derive(Clone)]
pub struct ServerRedis;

impl ServerRedis {
    pub fn start(argv: Vec<String>) -> Result<(), ErrorStruct> {
        // ################## 1° Initialization structures: BASIC ELEMENTS ##################
        let config = RedisConfig::parse_config(argv)?;
        let listener = ListenerProcessor::new_tcp_listener(&config)?;

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
        // and
        // ################## 6° Initialization structures: Notifier ##################
        let server_redis =
            ServerRedisAtributes::new(Arc::clone(&config), status_listener.clone(), shared_clients);

        let notifier = Notifier::new(
            sender_log.clone(),
            command_delegator_sender,
            status_listener,
            server_redis.get_addr()?,
        );
        let database = Database::new(notifier.clone());
        let runnables_database = RunnablesMap::<Database>::database();
        let runnables_server = RunnablesMap::<ServerRedisAtributes>::server();

        // ################## 7° Initialization structures: STRUCTS WITH THREADS ##################
        let mut log_center = LogCenter::new(
            sender_log,
            receiver,
            Arc::clone(&config),
            FileManager::new(),
        )?;

        let mut command_delegator =
            CommandDelegator::start(command_delegator_recv, commands_map, notifier.clone())?;
        let mut command_sub_delegator_databse = CommandSubDelegator::start::<Database>(
            rcv_cmd_dat,
            runnables_database,
            database,
            notifier.clone(),
        )?;
        let mut command_sub_delegator_server_atributes =
            CommandSubDelegator::start::<ServerRedisAtributes>(
                rcv_cmd_sv,
                runnables_server,
                server_redis.clone(),
                notifier.clone(),
            )?;

        let mut collector = GarbageCollector::start(snd_cmd_dat_garbage, 4, 20, notifier.clone());

        /*let quit_notifier = Mutex::new(notifier.clone());
        let quit: JoinHandle<Result<(), ErrorStruct>> = thread::spawn(move ||{
            for line in stdin().lock().lines() {
                match line {
                    Ok(line) => {
                        if line.contains("q") || line.contains("quit") || line.contains("exit") || line.contains("shutdown")  {
                            quit_notifier.lock().map_err(|_| {
                                ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::ShutdownServer))
                            })?.force_shutdown_server("Shutdown by stdin console of server".to_string()); break;
                        }
                    },
                    Err(e) => panic!("{}", e),
                }
            }
            Ok(())
        });*/
        // ################## ListenerProcessor ##################

        ListenerProcessor::incoming(listener, server_redis, notifier);

        // ################## FINISH SERVER ##################
        command_delegator.join()?;
        collector.join()?;
        command_sub_delegator_databse.join()?;
        command_sub_delegator_server_atributes.join()?;
        drop_shared_clients.lock().unwrap().join()?;
        log_center.join()?;
        //quit.join();

        Ok(())
    }
}
