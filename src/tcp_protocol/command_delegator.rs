use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
use crate::tcp_protocol::BoxedCommand;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::{collections::HashMap, thread};

use super::{RawCommand, Response};

use crate::joinable::Joinable;
use crate::messages::redis_messages;
use crate::messages::redis_messages::command_not_found;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::ErrorStruct;
use crate::tcp_protocol::close_thread;

pub struct CommandsMap {
    channel_map: HashMap<String, Vec<Option<Sender<RawCommand>>>>,
}

impl CommandsMap {
    pub fn kill_senders(&mut self) {
        self.channel_map.iter_mut().for_each(|x| {
            let senders = x.1;
            senders.iter_mut().for_each(|x| {
                let _ = x.take();
            })
        })
    }

    pub fn new(channel_map: HashMap<String, Vec<Option<Sender<RawCommand>>>>) -> CommandsMap {
        CommandsMap { channel_map }
    }

    pub fn get(&self, string: &str) -> Option<&Vec<Option<Sender<RawCommand>>>> {
        self.channel_map.get(string)
    }

    pub fn default() -> (CommandsMap, Receiver<RawCommand>, Receiver<RawCommand>) {
        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let (snd_cmd_server, rcv_cmd_server): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let mut channel_map: HashMap<String, Vec<Option<Sender<RawCommand>>>> = HashMap::new();
        channel_map.insert(String::from("set"), vec![Some(snd_cmd_dat.clone())]);
        channel_map.insert(String::from("get"), vec![Some(snd_cmd_dat.clone())]);
        channel_map.insert(String::from("strlen"), vec![Some(snd_cmd_dat)]);
        channel_map.insert(
            String::from("config get"),
            vec![Some(snd_cmd_server.clone())],
        );
        channel_map.insert(
            String::from("config set"),
            vec![Some(snd_cmd_server.clone())],
        );
        channel_map.insert(
            String::from("clear_client"),
            vec![Some(snd_cmd_server.clone())],
        );
        channel_map.insert(
            String::from("notify_monitors"),
            vec![Some(snd_cmd_server.clone())],
        );
        channel_map.insert(String::from("shutdown"), vec![Some(snd_cmd_server.clone())]);
        channel_map.insert(String::from("monitor"), vec![None]);
        channel_map.insert(
            String::from("subscribe"),
            vec![None, Some(snd_cmd_server.clone())],
        );
        channel_map.insert(
            String::from("unsubscribe"),
            vec![None, Some(snd_cmd_server.clone())],
        );
        channel_map.insert(String::from("publish"), vec![Some(snd_cmd_server)]);

        (CommandsMap { channel_map }, rcv_cmd_dat, rcv_cmd_server)
    }
}

pub struct CommandDelegator {
    join: Option<JoinHandle<Result<(), ErrorStruct>>>,
    sender: Sender<Option<RawCommand>>,
}

/// Interprets commands and delegates tasks

impl Joinable<()> for CommandDelegator {
    fn join(&mut self) -> Result<(), ErrorStruct> {
        println!("BYE COMMAND DELEGATOR!");

        let _ = self.sender.send(None);

        /*match self.sender.send(None) {
            Ok(()) => { /* Delegator has been closed right now*/ }
            Err(_) => { /* Delegator is already closed */ }
        }*/

        close_thread(self.join.take(), "Command Delegator")
    }
}

impl CommandDelegator {
    pub fn start(
        sender: Sender<Option<RawCommand>>,
        command_delegator_recv: Receiver<Option<RawCommand>>,
        commands_map: Arc<Mutex<CommandsMap>>,
    ) -> Result<Self, ErrorStruct> {
        let builder = thread::Builder::new().name("Command Delegator".into());

        let handler = builder
            .spawn(move || CommandDelegator::init(command_delegator_recv, commands_map))
            .map_err(|_| {
                ErrorStruct::from(redis_messages::init_failed(
                    "Command Delegator",
                    ErrorSeverity::ShutdownServer,
                ))
            })?;

        Ok(Self {
            join: Some(handler),
            sender,
        })
    }

    fn init(
        command_delegator_recv: Receiver<Option<RawCommand>>,
        commands_map: Arc<Mutex<CommandsMap>>,
    ) -> Result<(), ErrorStruct> {
        for packed_raw_command in command_delegator_recv.iter() {
            if let Some(raw_command) = packed_raw_command {
                let default = String::from("UNKNOWN");
                let command_type: &str = raw_command.0.get(0).clone().unwrap_or(&default);
                if let Some(command_dest) = commands_map.lock().unwrap().get(command_type) {
                    is_critical(delegate_jobs(raw_command, command_dest))?;
                } else {
                    let error = command_not_found(command_type.to_string(), raw_command.0);
                    is_critical(raw_command.1.send(Err(error)).map_err(|_| {
                        ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::Comunicate))
                    }))?;
                }
            } else {
                break;
            }
        }

        Ok(())
    }
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

fn delegate_jobs(
    raw_command: RawCommand,
    sender_list: &[Option<Sender<RawCommand>>],
) -> Result<(), ErrorStruct> {
    for sender in sender_list.iter() {
        let raw_command_clone = clone_raw_command(&raw_command);
        if let Some(snd_struct) = sender.as_ref() {
            //Case SOME: El comando se envia al subdelegator indicado
            snd_struct.send(raw_command_clone).map_err(|_| {
                ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::ShutdownServer))
            })?;
        } else {
            //Case NONE: El comando se ejecuta sobre el client status
            case_client_status(
                raw_command_clone.0,
                raw_command_clone.1,
                raw_command_clone.2,
            )?;
        }
    }

    Ok(())
}

fn case_client_status(
    command_buffer: Vec<String>,
    response_sender: Sender<Response>,
    client_status: Arc<Mutex<ClientFields>>,
) -> Result<(), ErrorStruct> {
    let review = client_status
        .lock()
        .map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "client_status",
                ErrorSeverity::CloseClient,
            ))
        })?
        .review_command(&command_buffer);

    match review {
        Ok(allowed_command) => {
            run_command(
                allowed_command,
                command_buffer,
                response_sender,
                client_status,
            )?;
        }
        Err(error) => {
            send_response(response_sender, Err(error))?;
            return Err(ErrorStruct::from(redis_messages::normal_error()));
        }
    }

    Ok(())
}

fn run_command(
    allowed_command: Option<Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>>,
    command_buffer: Vec<String>,
    response_sender: Sender<Response>,
    client_status: Arc<Mutex<ClientFields>>,
) -> Result<(), ErrorStruct> {
    if let Some(runnable) = allowed_command {
        let result = runnable.run(command_buffer, &mut Arc::clone(&client_status));
        send_response(response_sender, result.clone())?;
        result.map(|_| ())
    } else {
        Err(ErrorStruct::from(redis_messages::normal_error()))
    }
}

fn send_response(response_sender: Sender<Response>, response: Response) -> Result<(), ErrorStruct> {
    response_sender
        .send(response)
        .map_err(|_| ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::CloseClient)))
}

fn clone_raw_command(raw_command: &RawCommand) -> RawCommand {
    (
        clone_command_vec(&raw_command.0),
        raw_command.1.clone(),
        Arc::clone(&raw_command.2),
    )
}

fn clone_command_vec(command_vec: &[String]) -> Vec<String> {
    let mut clone = Vec::new();
    for word in command_vec.iter() {
        clone.push(String::from(word));
    }
    clone
}

#[cfg(test)]
pub mod test_command_delegator {

    use crate::tcp_protocol::command_subdelegator::CommandSubDelegator;
    use crate::tcp_protocol::BoxedCommand;
    use crate::vec_strings;
    use crate::{
        database::Database,
        native_types::{RError, RedisType},
    };
    use std::sync::mpsc;
    use std::sync::Arc;

    use super::*;
    use crate::commands::lists::lpop::LPop;
    use crate::commands::lists::lpush::LPush;
    use crate::commands::lists::lset::Lset;
    use crate::tcp_protocol::runnables_map::RunnablesMap;

    #[test]
    fn test01_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Arc<BoxedCommand<Database>>> = HashMap::new();
        map.insert(String::from("lpush"), Arc::new(Box::new(LPush)));
        map.insert(String::from("lpop"), Arc::new(Box::new(LPop)));
        map.insert(String::from("lset"), Arc::new(Box::new(Lset)));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _database_command_delegator =
            CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Vec<Option<Sender<RawCommand>>>> = HashMap::new();
        channel_map.insert(String::from("lpush"), vec![Some(snd_cmd_dat.clone())]);
        channel_map.insert(String::from("lpop"), vec![Some(snd_cmd_dat.clone())]);
        channel_map.insert(String::from("lset"), vec![Some(snd_cmd_dat.clone())]);

        let commands_map = CommandsMap::new(channel_map);
        let c_commands_map = Arc::new(Mutex::new(commands_map));

        let (snd_test_cmd, rcv_test_cmd): (
            Sender<Option<RawCommand>>,
            Receiver<Option<RawCommand>>,
        ) = mpsc::channel();

        let mut command_delegator = CommandDelegator::start(
            snd_test_cmd.clone(),
            rcv_test_cmd,
            Arc::clone(&c_commands_map),
        )
        .unwrap();

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd
            .send(Some((
                buffer_mock,
                snd_dat_test,
                Arc::new(Mutex::new(ClientFields::default())),
            )))
            .unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1.unwrap(), ":4\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec![
            "lset".to_string(),
            "key".to_string(),
            "0".to_string(),
            "breaking".to_string(),
        ];
        snd_test_cmd
            .send(Some((
                buffer_mock,
                snd_dat_test,
                Arc::new(Mutex::new(ClientFields::default())),
            )))
            .unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1.unwrap(), "+OK\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpop", "key", "4"];
        snd_test_cmd
            .send(Some((
                buffer_mock,
                snd_dat_test,
                Arc::new(Mutex::new(ClientFields::default())),
            )))
            .unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(
            response1.unwrap(),
            "*4\r\n$8\r\nbreaking\r\n$2\r\nmy\r\n$3\r\nnew\r\n$9\r\ndelegator\r\n".to_string()
        );

        c_commands_map.lock().unwrap().kill_senders();
        drop(snd_test_cmd);
        drop(snd_cmd_dat);

        command_delegator.join().unwrap();
    }

    #[test]
    fn test02_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Arc<BoxedCommand<Database>>> = HashMap::new();
        map.insert(String::from("lpop"), Arc::new(Box::new(LPop)));
        map.insert(String::from("lset"), Arc::new(Box::new(Lset)));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _database_command_delegator =
            CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Vec<Option<Sender<RawCommand>>>> = HashMap::new();
        channel_map.insert(String::from("lpop"), vec![Some(snd_cmd_dat.clone())]);
        channel_map.insert(String::from("lset"), vec![Some(snd_cmd_dat.clone())]);

        let commands_map = CommandsMap::new(channel_map);
        let c_commands_map = Arc::new(Mutex::new(commands_map));

        let (snd_test_cmd, rcv_test_cmd): (
            Sender<Option<RawCommand>>,
            Receiver<Option<RawCommand>>,
        ) = mpsc::channel();

        let mut command_delegator = CommandDelegator::start(
            snd_test_cmd.clone(),
            rcv_test_cmd,
            Arc::clone(&c_commands_map),
        )
        .unwrap();

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<Response>, Receiver<Response>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd
            .send(Some((
                buffer_mock,
                snd_dat_test,
                Arc::new(Mutex::new(ClientFields::default())),
            )))
            .unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(RError::encode(response1.unwrap_err()), "-ERR unknown command \'lpush\', with args beginning with: \'lpush\', \'key\', \'delegator\', \'new\', \'my\', \'testing\', \r\n".to_string());
        c_commands_map.lock().unwrap().kill_senders();
        drop(snd_test_cmd);
        drop(snd_cmd_dat);
        command_delegator.join().unwrap();
    }
}
