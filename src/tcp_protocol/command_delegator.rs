use std::sync::mpsc::{self, Receiver, Sender};
use std::{collections::HashMap, thread};

use crate::native_types::{ErrorStruct, RError, RedisType};

use super::SendersRedis;
pub struct CommandsMap {
    channel_map: HashMap<String, Sender<(Vec<String>, Sender<String>)>>,
    channel_map_server: HashMap<String, Sender<(Vec<String>, Sender<String>)>>,
}

impl CommandsMap {
    pub fn new(channel_map: HashMap<String, Sender<(Vec<String>, Sender<String>)>>) -> CommandsMap {
        CommandsMap {
            channel_map,
            channel_map_server: HashMap::new(),
        }
    }

    pub fn get(&self, string: &str) -> Option<&Sender<(Vec<String>, Sender<String>)>> {
        self.channel_map.get(string)
    }

    pub fn get_for_server(&self, vec: &[String]) -> Option<&Sender<(Vec<String>, Sender<String>)>> {
        let mut cmd = vec.get(0).unwrap_or(&" ".to_string()).to_string();
        cmd.push(' ');
        cmd.push_str(&vec.get(1).unwrap_or(&" ".to_string()).to_string());
        self.channel_map_server.get(&cmd)
    }

    pub fn default() -> (
        CommandsMap,
        Receiver<(Vec<String>, Sender<String>)>,
        Receiver<(Vec<String>, Sender<String>)>,
    ) {
        let (snd_cmd_dat, rcv_cmd_dat): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = mpsc::channel();

        let (snd_cmd_server, rcv_cmd_server): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = mpsc::channel();

        let mut channel_map: HashMap<String, Sender<(Vec<String>, Sender<String>)>> =
            HashMap::new();
        let mut channel_map_server: HashMap<String, Sender<(Vec<String>, Sender<String>)>> =
            HashMap::new();
        channel_map.insert(String::from("set"), snd_cmd_dat.clone());
        channel_map.insert(String::from("get"), snd_cmd_dat.clone());
        channel_map.insert(String::from("strlen"), snd_cmd_dat);
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
        command_delegator_recv: Receiver<(Vec<String>, Sender<String>)>,
        commands_map: CommandsMap,
    ) -> Result<(), ErrorStruct> {
        let _command_handler = thread::spawn(move || {
            for (command_input_user, sender_to_client) in command_delegator_recv.iter() {
                let command_type = &command_input_user[0];

                if let Some(command_dest) = commands_map.get(command_type) {
                    let _ = command_dest.send((command_input_user, sender_to_client));
                } else if let Some(command_dest) = commands_map.get_for_server(&command_input_user)
                {
                    let _ = command_dest.send((command_input_user, sender_to_client));
                } else {
                    let error =
                        ErrorStruct::new("ERR".to_string(), "command does not exist".to_string());
                    sender_to_client.send(RError::encode(error)).unwrap(); // CHECK UNWRAP L8R
                }
            }
        });
        Ok(())
    }

    /*
    pub fn new_update(
        command_delegator_recv: Receiver<(Vec<String>, Sender<String>)>,
        runnables: RunnablesMap,
        database: Arc<Mutex<Database>>
    ) -> Self {
        let _ = thread::Builder::new().name("Command Delegator".to_string()).spawn(move || {
            println!("{:?}",thread::current());
            for (mut command_input_user, sender_to_client) in command_delegator_recv.iter() {

                if let Some(runnable_command) = runnables.get(&command_input_user[0]) { // TODO: => "set" vs "config set"
                    command_input_user.remove(0); // ["set", "key", "value"]
                    let command_str: Vec<&str> = command_input_user.iter().map(|s| s.as_ref()).collect(); // TODO: => cambiar a Vec<String>
                    match runnable_command.run(command_str, &mut database.lock().unwrap()) {
                        Ok(encoded_resp) => sender_to_client.send(encoded_resp).unwrap(),
                        Err(err) => sender_to_client.send(RError::encode(err)).unwrap(),
                    };
                } else {
                    let error =
                        ErrorStruct::new("ERR_NEW".to_string(), "command does not exist".to_string());
                    sender_to_client.send(RError::encode(error));
                }


            }
        });
        CommandDelegator {}
    }
    */
}

#[cfg(test)]
pub mod test_command_delegator {

    use crate::database::Database;
    use crate::tcp_protocol::database_command_delegator::DatabaseCommandDelegator;
    use std::sync::mpsc;

    use super::*;
    use crate::commands::lists::lpop::LPop;
    use crate::commands::lists::lpush::LPush;
    use crate::commands::lists::lset::Lset;
    use crate::commands::Runnable;
    use crate::tcp_protocol::runnables_map::RunnablesMap;

    #[test]
    fn test01_lpush_lpop_lset() {
        // ARRANGE

        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("lpush"), Box::new(LPush));
        map.insert(String::from("lpop"), Box::new(LPop));
        map.insert(String::from("lset"), Box::new(Lset));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = mpsc::channel();

        let _database_command_delegator =
            DatabaseCommandDelegator::start(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<(Vec<String>, Sender<String>)>> =
            HashMap::new();
        channel_map.insert(String::from("lpush"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_test_cmd, rcv_test_cmd): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec![
            "lpush".to_string(),
            "key".to_string(),
            "delegator".to_string(),
            "new".to_string(),
            "my".to_string(),
            "testing".to_string(),
        ];
        snd_test_cmd.send((buffer_vec_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, ":4\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec![
            "lset".to_string(),
            "key".to_string(),
            "0".to_string(),
            "breaking".to_string(),
        ];
        snd_test_cmd.send((buffer_vec_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "+OK\r\n".to_string());

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec!["lpop".to_string(), "key".to_string(), "4".to_string()];
        snd_test_cmd.send((buffer_vec_mock, snd_dat_test)).unwrap();

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

        let (snd_cmd_dat, rcv_cmd_dat): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = mpsc::channel();

        let _database_command_delegator =
            DatabaseCommandDelegator::start(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<(Vec<String>, Sender<String>)>> =
            HashMap::new();
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_test_cmd, rcv_test_cmd): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = mpsc::channel();

        let _command_delegator = CommandDelegator::start(rcv_test_cmd, commands_map);

        // ACT

        let (snd_dat_test, rcv_dat_test): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec![
            "lpush".to_string(),
            "key".to_string(),
            "delegator".to_string(),
            "new".to_string(),
            "my".to_string(),
            "testing".to_string(),
        ];
        snd_test_cmd.send((buffer_vec_mock, snd_dat_test)).unwrap();

        // ASSERT

        let response1 = rcv_dat_test.recv().unwrap();
        assert_eq!(response1, "-ERR command does not exist\r\n".to_string());
    }
}
