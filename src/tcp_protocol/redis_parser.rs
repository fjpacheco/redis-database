
use std::{io::BufRead, thread};
use std::sync::mpsc::{Receiver,Sender};

use crate::native_types::{RArray, RError, RedisType};

pub struct RedisParser {
    redis_handler: thread::JoinHandle<()>,
}

/// Interprets commands and checks if they are able to be sent to the DatabaseCommandDelegator

impl RedisParser {
    pub fn new(client_handler_recv: Receiver<(Box<dyn BufRead + Send + Sync>, Sender<String>)>, command_delegator_sender: Sender<(Vec<String>,Sender<String>)>) -> Self {
        let redis_handler = thread::spawn(move || {
            for (mut buffer,resp_sender) in client_handler_recv.iter() {
                let command_delegator_sender_cl = command_delegator_sender.clone();
                match RArray::decode(&mut buffer) {
                    Ok(command) => {
                        command_delegator_sender_cl.send((command,resp_sender));
                    },
                    Err(err) => {
                        resp_sender.send(RError::encode(err));
                    },
                }
            }
        });
        RedisParser{redis_handler}
    }
}

#[cfg(test)]
pub mod test_redis_parser {

    use std::io::BufReader;
use crate::commands::lists::rpop::RPop;
    use crate::{tcp_protocol::command_delegator::CommandDelegator, commands::lists::lpush::LPush};
    use crate::tcp_protocol::command_delegator::CommandsMap;
    use crate::database::Database;
    use crate::tcp_protocol::database_command_delegator::DatabaseCommandDelegator;
    use std::sync::mpsc::Receiver;
    use std::sync::mpsc::Sender;
    use crate::tcp_protocol::database_command_delegator::RunnablesMap;
    use crate::commands::Runnable;
    use std::{collections::HashMap, sync::mpsc};

    use super::*;
    #[test]
    fn test01_lpush_lpop_lset<'a>() {

// ARRANGE
        
        let mut map: HashMap<String, Box<dyn Runnable + Send + Sync>> = HashMap::new();
        map.insert(String::from("lpush"), Box::new(LPush));
        map.insert(String::from("lpop"), Box::new(RPop));
        map.insert(String::from("lset"), Box::new(LPush));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (snd_cmd_dat, rcv_cmd_dat): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = mpsc::channel();

        let _database_command_delegator =
            DatabaseCommandDelegator::new(rcv_cmd_dat, runnables_map, database);

        let mut channel_map: HashMap<String, Sender<(Vec<String>, Sender<String>)>> = HashMap::new();
        channel_map.insert(String::from("lpush"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lpop"), snd_cmd_dat.clone());
        channel_map.insert(String::from("lset"), snd_cmd_dat.clone());

        let commands_map = CommandsMap::new(channel_map);

        let (snd_par_cmd, rcv_par_cmd): (
            Sender<(Vec<String>, Sender<String>)>,
            Receiver<(Vec<String>, Sender<String>)>,
        ) = mpsc::channel();

        let _command_delegator = CommandDelegator::new(rcv_par_cmd, commands_map);

        let (snd_test_par, rcv_test_par): (
            Sender<(Box<dyn BufRead + Send + Sync>,Sender<String>)>,
            Receiver<(Box<dyn BufRead + Send + Sync>,Sender<String>)>,
        ) = mpsc::channel();

        let _redis_parser = RedisParser::new(rcv_test_par, snd_par_cmd);

        // ACT

        let (snd_par_test, rcv_par_test): (
            Sender<String>,
            Receiver<String>,
        ) = mpsc::channel();
        let buffer_vec_mock = String::from("asdasdsad");
        let lil_box: Box<dyn BufRead + Send + Sync + 'a > = Box::new(BufReader::new(buffer_vec_mock.as_bytes()));
        snd_test_par.send((lil_box, snd_par_test)).unwrap();

        let response1 = rcv_par_test.recv().unwrap();

    }

}