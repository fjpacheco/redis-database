use crate::tcp_protocol::BoxedCommand;
use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{collections::HashMap, thread};

use super::RawCommand;

use crate::messages::redis_messages::command_not_found;
use crate::native_types::{ErrorStruct, RError, RedisType};

/*
pub struct CommandsMap {
    channel_map: HashMap<String, Sender<RawCommand>>,
    channel_map_server: HashMap<String, Sender<RawCommand>>,
}

impl CommandsMap {
    pub fn new(channel_map: HashMap<String, Sender<RawCommand>>) -> CommandsMap {
        CommandsMap {
            channel_map,
            channel_map_server: HashMap::new(),
        }
    }

    pub fn get(&self, string: &str) -> Option<&Sender<RawCommand>> {
        self.channel_map.get(string)
    }

    pub fn get_for_server(&self, string: &str) -> Option<&Sender<RawCommand>> {
        self.channel_map_server.get(string)
    }

    pub fn default() -> (CommandsMap, Receiver<RawCommand>, Receiver<RawCommand>) {
        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let (snd_cmd_server, rcv_cmd_server): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let mut channel_map: HashMap<String, Sender<RawCommand>> = HashMap::new();
        let mut channel_map_server: HashMap<String, Sender<RawCommand>> = HashMap::new();
        channel_map.insert(String::from("set"), snd_cmd_dat.clone());
        channel_map.insert(String::from("get"), snd_cmd_dat.clone());
        channel_map.insert(String::from("strlen"), snd_cmd_dat);
        channel_map_server.insert(String::from("shutdown"), snd_cmd_server.clone());
        channel_map_server.insert(String::from("config set"), snd_cmd_server);

        (
            CommandsMap {
                channel_map,
                channel_map_server,
            },
            rcv_cmd_dat,
            rcv_cmd_server,
        )
    }
}

pub struct CommandDelegator;

/// Interprets commands and delegates tasks

impl CommandDelegator {
    pub fn start(
        command_delegator_recv: Receiver<RawCommand>,
        commands_map: CommandsMap,
    ) -> Result<(), ErrorStruct> {
        let builder = thread::Builder::new().name("Command Delegator".into());

        let command_delegator_handler = builder.spawn(move || {
            for (command_input_user, sender_to_client) in command_delegator_recv.iter() {
                let mut command_type = command_input_user[0].to_string();
                if command_type.contains("config") {
                    command_type =
                        command_type.to_owned() + " " + &command_input_user[1].to_string();
                }

                if let Some(command_dest) = commands_map.get(&command_type) {
                    let _ = command_dest.send((command_input_user, sender_to_client));
                } else {
                    let error = command_not_found(command_type.to_string(), command_input_user);
                    sender_to_client.send(RError::encode(error)).unwrap();
                }
            }
        });

        match command_delegator_handler {
            Ok(_) => Ok(()),
            Err(item) => Err(ErrorStruct::new(
                "ERR_THREAD_BUILDER".into(),
                format!("{}", item),
            )),
        }
    }
}

#[cfg(test)]
pub mod test_command_delegator {

    use std::sync::Arc;
use crate::database::Database;
    use crate::tcp_protocol::command_subdelegator::CommandSubDelegator;
    use crate::vec_strings;
    use std::sync::mpsc;

    use super::*;
    use crate::commands::lists::lpop::LPop;
    use crate::commands::lists::lpush::LPush;
    use crate::commands::lists::lset::Lset;
    use crate::commands::Runnable;
    use crate::tcp_protocol::runnables_map::RunnablesMap;

    /*
    #[test]
    fn test01_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("lpush"), Box::new(LPush));
        map.insert(String::from("lpop"), Box::new(LPop));
        map.insert(String::from("lset"), Box::new(Lset));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _database_command_delegator =
            CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<RawCommand>> = HashMap::new();
        channel_map.insert(String::from("lpush"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_test_cmd, rcv_test_cmd): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, ":4\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec![
            "lset".to_string(),
            "key".to_string(),
            "0".to_string(),
            "breaking".to_string(),
        ];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "+OK\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpop", "key", "4"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(
            response1,
            "*4\r\n$8\r\nbreaking\r\n$2\r\nmy\r\n$3\r\nnew\r\n$9\r\ndelegator\r\n".to_string()
        );
    }

    #[test]
    fn test02_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("lpop"), Box::new(LPop));
        map.insert(String::from("lset"), Box::new(Lset));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _database_command_delegator =
            CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<RawCommand>> = HashMap::new();
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_test_cmd, rcv_test_cmd): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "-ERR unknown command \'lpush\', with args beginning with: \'lpush\', \'key\', \'delegator\', \'new\', \'my\', \'testing\', \r\n".to_string());
    }
    */

    #[test]
    fn test01_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Arc<Box<dyn Runnable<Database> + Send + Sync>>> = HashMap::new();
        map.insert(String::from("lpush"), Arc::new(Box::new(LPush)));
        map.insert(String::from("lpop"), Arc::new(Box::new(LPop)));
        map.insert(String::from("lset"), Arc::new(Box::new(Lset)));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _database_command_delegator =
            CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<RawCommand>> = HashMap::new();
        channel_map.insert(String::from("lpush"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_test_cmd, rcv_test_cmd): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, ":4\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec![
            "lset".to_string(),
            "key".to_string(),
            "0".to_string(),
            "breaking".to_string(),
        ];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "+OK\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpop", "key", "4"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(
            response1,
            "*4\r\n$8\r\nbreaking\r\n$2\r\nmy\r\n$3\r\nnew\r\n$9\r\ndelegator\r\n".to_string()
        );
    }

    #[test]
    fn test02_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Arc<Box<dyn Runnable<Database> + Send + Sync>>> = HashMap::new();
        map.insert(String::from("lpop"), Arc::new(Box::new(LPop)));
        map.insert(String::from("lset"), Arc::new(Box::new(Lset)));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _database_command_delegator =
            CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<RawCommand>> = HashMap::new();
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_test_cmd, rcv_test_cmd): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "-ERR unknown command \'lpush\', with args beginning with: \'lpush\', \'key\', \'delegator\', \'new\', \'my\', \'testing\', \r\n".to_string());
    }
}
*/



pub struct CommandsMap {
    channel_map: HashMap<String, Vec<Option<Sender<RawCommand>>>>,
    channel_map_server: HashMap<String, Sender<RawCommand>>,
}

impl CommandsMap {
    pub fn new(channel_map: HashMap<String, Vec<Option<Sender<RawCommand>>>>) -> CommandsMap {
        CommandsMap {
            channel_map,
            channel_map_server: HashMap::new(),
        }
    }

    pub fn get(&self, string: &str) -> Option<&Vec<Option<Sender<RawCommand>>>> {
        self.channel_map.get(string)
    }

    pub fn get_for_server(&self, string: &str) -> Option<&Sender<RawCommand>> {
        self.channel_map_server.get(string)
    }

    pub fn default() -> (CommandsMap, Receiver<RawCommand>, Receiver<RawCommand>) {
        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let (snd_cmd_server, rcv_cmd_server): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let mut channel_map: HashMap<String, Vec<Option<Sender<RawCommand>>>> = HashMap::new();
        let mut channel_map_server: HashMap<String, Sender<RawCommand>> = HashMap::new();
        channel_map.insert(String::from("set"), vec![Some(snd_cmd_dat.clone())]);
        channel_map.insert(String::from("get"), vec![Some(snd_cmd_dat.clone())]);
        channel_map.insert(String::from("strlen"), vec![Some(snd_cmd_dat)]);
        channel_map_server.insert(String::from("shutdown"), snd_cmd_server.clone());
        channel_map_server.insert(String::from("config set"), snd_cmd_server);

        (
            CommandsMap {
                channel_map,
                channel_map_server,
            },
            rcv_cmd_dat,
            rcv_cmd_server,
        )
    }
}

pub struct CommandDelegator;

/// Interprets commands and delegates tasks

impl CommandDelegator {
    pub fn start(
        command_delegator_recv: Receiver<RawCommand>,
        commands_map: CommandsMap,
    ) -> Result<(), ErrorStruct> {
        let builder = thread::Builder::new().name("Command Delegator".into());

        let command_delegator_handler = builder.spawn(move || {
            for raw_command in command_delegator_recv.iter() {
                let command_type = raw_command.0.get(0).unwrap();
                if let Some(command_dest) = commands_map.get(command_type) {
                    delegate_jobs(raw_command, command_dest);
                } else {
                    let error = command_not_found(command_type.to_string(), raw_command.0);
                    raw_command.1.send(RError::encode(error)).unwrap();
                }
            }
        });

        match command_delegator_handler {
            Ok(_) => Ok(()),
            Err(item) => Err(ErrorStruct::new(
                "ERR_THREAD_BUILDER".into(),
                format!("{}", item),
            )),
        }
    }
}

fn delegate_jobs(raw_command: RawCommand, sender_list: &Vec<Option<Sender<RawCommand>>>) {
    
    for sender in sender_list.iter() {
        let raw_command_clone = clone_raw_command(&raw_command);
        if let Some(snd_struct) = sender.as_ref() {
            //Case SOME: El comando se envia al subdelegator indicado
            snd_struct.send(raw_command_clone).unwrap();
        } else {
            //Case NONE: El comando se ejecuta sobre el client status
            let command_buffer = raw_command_clone.0;
            let response_sender = raw_command_clone.1;
            let client_status = raw_command_clone.2;

            let review = client_status.lock().unwrap().review_command(&command_buffer);
            
            match review {
                Ok(allowed_command) => {
                    if let Some(runnable) = allowed_command {
                        response_sender.send(run_command(runnable, command_buffer, client_status)).unwrap();
                    } else {
                        //error
                        break;
                        //FIXME :(
                    }
                },
                Err(error) => {
                    response_sender.send(RError::encode(error)).unwrap();
                    break;
                    //FIXME
                },
            }
        }
    }
}

fn run_command(runnable: Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>, command_buffer: Vec<String>, mut client: Arc<Mutex<ClientFields>>) -> String {
    match runnable.run(command_buffer, &mut client) {
        Ok(response) => response,
        Err(error) => RError::encode(error),
    }
}

fn clone_raw_command(raw_command: &RawCommand) -> RawCommand {
    (clone_command_vec(&raw_command.0), raw_command.1.clone(), Arc::clone(&raw_command.2))
}

fn clone_command_vec(command_vec: &Vec<String>) -> Vec<String> {
    let mut clone = Vec::new();
    for word in command_vec.iter(){ clone.push(String::from(word)); }
    clone
}



#[cfg(test)]
pub mod test_command_delegator {

    use std::sync::Arc;
use crate::database::Database;
    use crate::tcp_protocol::command_subdelegator::CommandSubDelegator;
    use crate::vec_strings;
    use std::sync::mpsc;

    use super::*;
    use crate::commands::lists::lpop::LPop;
    use crate::commands::lists::lpush::LPush;
    use crate::commands::lists::lset::Lset;
    use crate::commands::Runnable;
    use crate::tcp_protocol::runnables_map::RunnablesMap;

    /*
    #[test]
    fn test01_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("lpush"), Box::new(LPush));
        map.insert(String::from("lpop"), Box::new(LPop));
        map.insert(String::from("lset"), Box::new(Lset));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _database_command_delegator =
            CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<RawCommand>> = HashMap::new();
        channel_map.insert(String::from("lpush"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_test_cmd, rcv_test_cmd): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, ":4\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec![
            "lset".to_string(),
            "key".to_string(),
            "0".to_string(),
            "breaking".to_string(),
        ];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "+OK\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpop", "key", "4"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(
            response1,
            "*4\r\n$8\r\nbreaking\r\n$2\r\nmy\r\n$3\r\nnew\r\n$9\r\ndelegator\r\n".to_string()
        );
    }

    #[test]
    fn test02_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("lpop"), Box::new(LPop));
        map.insert(String::from("lset"), Box::new(Lset));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _database_command_delegator =
            CommandSubDelegator::start::<Database>(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<RawCommand>> = HashMap::new();
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_test_cmd, rcv_test_cmd): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd.send((buffer_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "-ERR unknown command \'lpush\', with args beginning with: \'lpush\', \'key\', \'delegator\', \'new\', \'my\', \'testing\', \r\n".to_string());
    }
    */

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

        let (snd_test_cmd, rcv_test_cmd): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd.send((buffer_mock, snd_dat_test, Arc::new(Mutex::new(ClientFields::default())))).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, ":4\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec![
            "lset".to_string(),
            "key".to_string(),
            "0".to_string(),
            "breaking".to_string(),
        ];
        snd_test_cmd.send((buffer_mock, snd_dat_test, Arc::new(Mutex::new(ClientFields::default())))).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "+OK\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpop", "key", "4"];
        snd_test_cmd.send((buffer_mock, snd_dat_test, Arc::new(Mutex::new(ClientFields::default())))).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(
            response1,
            "*4\r\n$8\r\nbreaking\r\n$2\r\nmy\r\n$3\r\nnew\r\n$9\r\ndelegator\r\n".to_string()
        );
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

        let (snd_test_cmd, rcv_test_cmd): (Sender<RawCommand>, Receiver<RawCommand>) =
            mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_mock = vec_strings!["lpush", "key", "delegator", "new", "my", "testing"];
        snd_test_cmd.send((buffer_mock, snd_dat_test, Arc::new(Mutex::new(ClientFields::default())))).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "-ERR unknown command \'lpush\', with args beginning with: \'lpush\', \'key\', \'delegator\', \'new\', \'my\', \'testing\', \r\n".to_string());
    }
}