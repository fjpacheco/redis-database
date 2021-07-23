use std::sync::{atomic::AtomicBool, mpsc::channel, Arc, Mutex};

use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
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
    client_list::ClientList, command_subdelegator::CommandSubDelegator, commands_map::CommandsMap,
    notifier::Notifier,
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
        let (snd_cmd_dat, rcv_cmd_dat) = channel();
        let (snd_cmd_sv, rcv_cmd_sv) = channel();
        let commands_map = CommandsMap::default(snd_cmd_dat.clone(), snd_cmd_sv.clone());

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
        let database = Database::new_from(Arc::clone(&config), notifier.clone())?;

        let c_database = Arc::new(Mutex::new(database));
        let runnables_database = RunnablesMap::<Arc<Mutex<Database>>>::database();
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
        let mut command_sub_delegator_databse = CommandSubDelegator::start::<Arc<Mutex<Database>>>(
            snd_cmd_dat.clone(),
            rcv_cmd_dat,
            runnables_database,
            Arc::clone(&c_database),
            notifier.clone(),
        )?;
        let mut command_sub_delegator_server_atributes =
            CommandSubDelegator::start::<ServerRedisAtributes>(
                snd_cmd_sv,
                rcv_cmd_sv,
                runnables_server,
                server_redis.clone(),
                notifier.clone(),
            )?;

        let mut collector = GarbageCollector::new(snd_cmd_dat, 1, 20, notifier.clone());

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
        c_database
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "database",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .take_snapshot()?;

        // ################## FINISH SERVER ##################
        command_delegator.join()?;
        collector.join()?;
        command_sub_delegator_databse.join()?;
        command_sub_delegator_server_atributes.join()?;
        drop_shared_clients
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "shared clients",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .join()?;
        log_center.join()?;
        //quit.join();

        Ok(())
    }
}
