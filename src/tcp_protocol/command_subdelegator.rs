use crate::communication::log_messages::LogMessage;
use crate::joinable::Joinable;
use crate::messages::redis_messages;
use crate::tcp_protocol::close_thread;
use crate::tcp_protocol::BoxedCommand;
use std::fmt::Display;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::ErrorStruct;
use crate::tcp_protocol::runnables_map::RunnablesMap;

use super::notifier::Notifier;
use super::{RawCommand, Response};
pub struct CommandSubDelegator {
    sender: Sender<Option<RawCommand>>,
    join: Option<JoinHandle<Result<(), ErrorStruct>>>,
    notifier: Notifier,
}
/// Interprets commands and delegates tasks

impl Joinable<()> for CommandSubDelegator {
    fn join(&mut self) -> Result<(), ErrorStruct> {
        let _ = self.sender.send(None);
        close_thread(
            self.join.take(),
            "Command Subdelegator",
            self.notifier.clone(),
        )
    }
}
impl CommandSubDelegator {
    pub fn start<T: 'static>(
        snd_cmd: Sender<Option<RawCommand>>,
        rcv_cmd: Receiver<Option<RawCommand>>,
        runnables_map: RunnablesMap<T>,
        data: T,
        notifier: Notifier,
    ) -> Result<Self, ErrorStruct>
    where
        T: Send + Sync + Display,
    {
        let builder = thread::Builder::new().name(format!("Command Sub-Delegator for {}", data));
        let c_notifier = notifier.clone();
        let command_sub_delegator_handler = builder
            .spawn(move || CommandSubDelegator::init(rcv_cmd, runnables_map, data, c_notifier))
            .map_err(|_| {
                ErrorStruct::from(redis_messages::init_failed(
                    "Command Subdelegator",
                    ErrorSeverity::ShutdownServer,
                ))
            })?;

        Ok(Self {
            sender: snd_cmd,
            join: Some(command_sub_delegator_handler),
            notifier,
        })
    }

    fn init<T: 'static>(
        rcv_cmd: Receiver<Option<RawCommand>>,
        runnables_map: RunnablesMap<T>,
        mut data: T,
        notifier: Notifier,
    ) -> Result<(), ErrorStruct>
    where
        T: Send + Sync + Display,
    {
        for packed_raw_command in rcv_cmd.iter() {
            if let Some((mut command_input_user, sender_to_client, _)) = packed_raw_command {
                let command_type = command_input_user.remove(0);
                if let Some(runnable_command) = runnables_map.get(&command_type) {
                    let err_critical = is_critical(run_command(
                        runnable_command,
                        command_input_user,
                        sender_to_client,
                        &mut data,
                    ));
                    if let Err(err) = err_critical {
                        if err.severity().eq(&Some(&ErrorSeverity::ShutdownServer)) {
                            notifier.force_shutdown_server(err.print_it());
                            return Err(err);
                        }
                    }
                } else {
                    let error = redis_messages::command_not_found(command_type, command_input_user);
                    if sender_to_client.send(Err(error)).is_err() {
                        notifier.send_log(LogMessage::channel_client_off())?;
                    }
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}

fn run_command<T: 'static>(
    runnable_command: Arc<BoxedCommand<T>>,
    command_input_user: Vec<String>,
    sender_to_client: Sender<Response>,
    data: &mut T,
) -> Result<(), ErrorStruct> {
    let result = runnable_command.run(command_input_user, data);

    sender_to_client
        .send(result.clone())
        .map_err(|_| ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::Comunicate)))?;

    result.map(|_| ())
}

fn is_critical(potential_error: Result<(), ErrorStruct>) -> Result<(), ErrorStruct> {
    /*
     * Lista de errores que lanza delegate_jobs():
     * - closed subdelegator channel -> Shutdown server
     * - closed client channel -> Nothing happens
     * - poisoned lock -> Shutdown server
     * - normal error -> Nothing happens
     */

    match potential_error {
        Ok(()) => Ok(()),
        Err(error) => check_severity(error),
    }
}

fn check_severity(error: ErrorStruct) -> Result<(), ErrorStruct> {
    if let Some(severity) = error.severity() {
        match severity {
            ErrorSeverity::ShutdownServer => Err(error),
            _ => Ok(()),
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
pub mod test_database_command_delegator {
    use crate::commands::create_notifier;
    use crate::commands::lists::rpop::RPop;
    use crate::commands::lists::rpush::RPush;
    use crate::commands::strings::get::Get;
    use crate::commands::strings::set::Set;
    use crate::commands::strings::strlen::Strlen;
    use crate::communication::log_messages::LogMessage;
    use crate::native_types::RError;
    use crate::native_types::RedisType;
    use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
    use crate::tcp_protocol::BoxedCommand;
    use crate::{commands::lists::llen::Llen, tcp_protocol::Response};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::sync::Mutex;

    use crate::database::Database;
    use crate::vec_strings;
    use std::{
        collections::HashMap,
        sync::mpsc::{self, Receiver, Sender},
    };

    use super::*;

    #[test]

    fn test01_set_get_strlen() {
        let mut map: HashMap<String, Arc<BoxedCommand<Database>>> = HashMap::new();
        map.insert(String::from("set"), Arc::new(Box::new(Set)));
        map.insert(String::from("get"), Arc::new(Box::new(Get)));
        map.insert(String::from("strlen"), Arc::new(Box::new(Strlen)));

        let runnables_map = RunnablesMap::new(map);

        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let database = Database::new(notifier.clone());

        let (tx1, rx1) = mpsc::channel();

        let _database_command_delegator_recv = CommandSubDelegator::start::<Database>(
            tx1.clone(),
            rx1,
            runnables_map,
            database,
            notifier.clone(),
        );

        let (tx2, rx2): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec_strings!["set", "key", "value"];
        tx1.send(Some((
            buffer_mock,
            tx2,
            Arc::new(Mutex::new(ClientFields::default())),
        )))
        .unwrap();

        let response1 = rx2.recv().unwrap();
        assert_eq!(response1.unwrap(), "+OK\r\n".to_string());

        let (tx3, rx3): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock_get = vec!["get".to_string(), "key".to_string()];
        tx1.send(Some((
            buffer_mock_get,
            tx3,
            Arc::new(Mutex::new(ClientFields::default())),
        )))
        .unwrap();

        let response2 = rx3.recv().unwrap();
        assert_eq!(response2.unwrap(), "$5\r\nvalue\r\n".to_string());

        let buffer_mock_strlen = vec_strings!["strlen", "key"];
        let (tx4, rx4): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        tx1.send(Some((
            buffer_mock_strlen,
            tx4,
            Arc::new(Mutex::new(ClientFields::default())),
        )))
        .unwrap();

        let response3 = rx4.recv().unwrap();
        assert_eq!(response3.unwrap(), ":5\r\n".to_string());
        drop(notifier);
        drop(tx1);
    }

    #[test]
    fn test02_get_command_does_not_exist() {
        let mut map: HashMap<String, Arc<BoxedCommand<Database>>> = HashMap::new();
        map.insert(String::from("set"), Arc::new(Box::new(Set)));
        let runnables_map = RunnablesMap::new(map);

        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let database = Database::new(notifier.clone());

        let (tx1, rx1) = mpsc::channel();

        let _database_command_delegator_recv = CommandSubDelegator::start::<Database>(
            tx1.clone(),
            rx1,
            runnables_map,
            database,
            notifier.clone(),
        );

        let (tx2, rx2): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec_strings!["set", "key", "value"];
        tx1.send(Some((
            buffer_mock,
            tx2,
            Arc::new(Mutex::new(ClientFields::default())),
        )))
        .unwrap();

        let response1 = rx2.recv().unwrap();
        assert_eq!(response1.unwrap(), "+OK\r\n".to_string());

        let (tx3, rx3): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock_get = vec_strings!["get", "key"];
        tx1.send(Some((
            buffer_mock_get,
            tx3,
            Arc::new(Mutex::new(ClientFields::default())),
        )))
        .unwrap();

        let response2 = rx3.recv().unwrap();
        assert_eq!(
            RError::encode(response2.unwrap_err()),
            "-ERR unknown command \'get\', with args beginning with: \'key\', \r\n".to_string()
        );
        drop(notifier);
        drop(tx1);
    }

    #[test]
    fn test03_rpush_rpop_llen() {
        let mut map: HashMap<String, Arc<BoxedCommand<Database>>> = HashMap::new();
        map.insert(String::from("rpush"), Arc::new(Box::new(RPush)));
        map.insert(String::from("rpop"), Arc::new(Box::new(RPop)));
        map.insert(String::from("llen"), Arc::new(Box::new(Llen)));
        let runnables_map = RunnablesMap::new(map);

        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let database = Database::new(notifier);

        let (tx1, rx1) = mpsc::channel();
        let (snd_log_test, _b): (Sender<Option<LogMessage>>, Receiver<Option<LogMessage>>) =
            mpsc::channel();

        let (snd_cmd_del_test, _c): (Sender<Option<RawCommand>>, Receiver<Option<RawCommand>>) =
            mpsc::channel();
        let notifier = Notifier::new(
            snd_log_test,
            snd_cmd_del_test,
            Arc::new(AtomicBool::new(false)),
            "test_addr".into(),
        );

        let _database_command_delegator_recv = CommandSubDelegator::start::<Database>(
            tx1.clone(),
            rx1,
            runnables_map,
            database,
            notifier.clone(),
        );
        let (tx2, rx2): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec![
            "rpush".to_string(),
            "key".to_string(),
            "value1".to_string(),
            "value2".to_string(),
            "value3".to_string(),
        ];
        tx1.send(Some((
            buffer_mock,
            tx2,
            Arc::new(Mutex::new(ClientFields::default())),
        )))
        .unwrap();

        let response1 = rx2.recv().unwrap();
        assert_eq!(response1.unwrap(), ":3\r\n".to_string());

        let (tx3, rx3): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec_strings!["rpop", "key", "2"];
        tx1.send(Some((
            buffer_mock,
            tx3,
            Arc::new(Mutex::new(ClientFields::default())),
        )))
        .unwrap();

        let response1 = rx3.recv().unwrap();
        assert_eq!(
            response1.unwrap(),
            "*2\r\n$6\r\nvalue3\r\n$6\r\nvalue2\r\n".to_string()
        );

        let (tx4, rx4): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec_strings!["llen", "value"];
        tx1.send(Some((
            buffer_mock,
            tx4,
            Arc::new(Mutex::new(ClientFields::default())),
        )))
        .unwrap();
        let response1 = rx4.recv().unwrap();
        assert_eq!(response1.unwrap(), ":0\r\n".to_string());

        drop(notifier);
        drop(tx1);
    }
}
