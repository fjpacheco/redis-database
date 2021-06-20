use crate::messages::redis_messages::command_not_found;
use crate::native_types::RError;
use crate::native_types::RedisType;
use std::fmt::Display;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::native_types::ErrorStruct;
use crate::tcp_protocol::runnables_map::RunnablesMap;

use super::{get_command_type, RawCommand};
pub struct CommandSubDelegator;
/// Interprets commands and delegates tasks

impl CommandSubDelegator {
    pub fn start<T: 'static>(
        rcv_cmd: Receiver<RawCommand>,
        runnables_map: RunnablesMap<T>,
        mut data: T,
    ) -> Result<(), ErrorStruct>
    where
        T: Send + Sync + Display,
    {
        let builder = thread::Builder::new().name(format!("Command Sub-Delegator for {}", data));

        let command_sub_delegator_handler = builder.spawn(move || {
            for (mut command_input_user, sender_to_client) in rcv_cmd.iter() {
                let command_type = get_command_type(&mut command_input_user);

                if let Some(runnable_command) = runnables_map.get(&command_type) {
                    // REMEMBER TO CHANGE THE TRAIT (MUST WORK WITH VEC<STRING>)
                    let command_str: Vec<&str> =
                        command_input_user.iter().map(|s| s.as_ref()).collect();
                    match runnable_command.run(command_str, &mut data) {
                        Ok(encoded_resp) => sender_to_client.send(encoded_resp).unwrap(),
                        Err(err) => sender_to_client.send(RError::encode(err)).unwrap(),
                    };
                } else {
                    let error = command_not_found(command_type, command_input_user);
                    sender_to_client.send(RError::encode(error)).unwrap();
                }
            }
        });

        match command_sub_delegator_handler {
            Ok(_) => Ok(()),
            Err(item) => Err(ErrorStruct::new(
                "ERR_THREAD_BUILDER".into(),
                format!("{}", item),
            )),
        }
    }
}

#[cfg(test)]
pub mod test_database_command_delegator {
    use crate::commands::lists::llen::Llen;
    use crate::commands::lists::rpop::RPop;
    use crate::commands::lists::rpush::RPush;
    use crate::commands::strings::get::Get;
    use crate::commands::strings::set::Set;
    use crate::commands::strings::strlen::Strlen;

    use crate::{commands::Runnable, database::Database};
    use std::{
        collections::HashMap,
        sync::mpsc::{self, Receiver, Sender},
    };

    use super::*;

    #[test]
    fn test01_set_get_strlen() {
        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("set"), Box::new(Set));
        map.insert(String::from("get"), Box::new(Get));
        map.insert(String::from("strlen"), Box::new(Strlen));

        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (tx1, rx1): (Sender<RawCommand>, Receiver<RawCommand>) = mpsc::channel();

        let _database_command_delegator_recv =
            CommandSubDelegator::start::<Database>(rx1, runnables_map, database);

        let (tx2, rx2): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec!["set".to_string(), "key".to_string(), "value".to_string()];
        tx1.send((buffer_vec_mock, tx2)).unwrap();

        let response1 = rx2.recv().unwrap();
        assert_eq!(response1, "+OK\r\n".to_string());

        let (tx3, rx3): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock_get = vec!["get".to_string(), "key".to_string()];
        tx1.send((buffer_vec_mock_get, tx3)).unwrap();

        let response2 = rx3.recv().unwrap();
        assert_eq!(response2, "$5\r\nvalue\r\n".to_string());

        let buffer_vec_mock_strlen = vec!["strlen".to_string(), "key".to_string()];
        let (tx4, rx4): (Sender<String>, Receiver<String>) = mpsc::channel();
        tx1.send((buffer_vec_mock_strlen, tx4)).unwrap();

        let response3 = rx4.recv().unwrap();
        assert_eq!(response3, ":5\r\n".to_string());
    }

    #[test]
    fn test02_get_command_does_not_exist() {
        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("set"), Box::new(Set));
        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (tx1, rx1): (Sender<RawCommand>, Receiver<RawCommand>) = mpsc::channel();

        let _database_command_delegator_recv =
            CommandSubDelegator::start::<Database>(rx1, runnables_map, database);

        let (tx2, rx2): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec!["set".to_string(), "key".to_string(), "value".to_string()];
        tx1.send((buffer_vec_mock, tx2)).unwrap();

        let response1 = rx2.recv().unwrap();
        assert_eq!(response1, "+OK\r\n".to_string());

        let (tx3, rx3): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock_get = vec!["get".to_string(), "key".to_string()];
        tx1.send((buffer_vec_mock_get, tx3)).unwrap();

        let response2 = rx3.recv().unwrap();
        assert_eq!(
            response2,
            "-ERR unknown command \'get\', with args beginning with: \'key\', \r\n".to_string()
        );
    }

    #[test]
    fn test03_rpush_rpop_llen() {
        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("rpush"), Box::new(RPush));
        map.insert(String::from("rpop"), Box::new(RPop));
        map.insert(String::from("llen"), Box::new(Llen));
        let runnables_map = RunnablesMap::new(map);

        let database = Database::new();

        let (tx1, rx1): (Sender<RawCommand>, Receiver<RawCommand>) = mpsc::channel();

        let _database_command_delegator_recv =
            CommandSubDelegator::start::<Database>(rx1, runnables_map, database);

        let (tx2, rx2): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec![
            "rpush".to_string(),
            "key".to_string(),
            "value1".to_string(),
            "value2".to_string(),
            "value3".to_string(),
        ];
        tx1.send((buffer_vec_mock, tx2)).unwrap();

        let response1 = rx2.recv().unwrap();
        assert_eq!(response1, ":3\r\n".to_string());

        let (tx3, rx3): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec!["rpop".to_string(), "key".to_string(), "2".to_string()];
        tx1.send((buffer_vec_mock, tx3)).unwrap();

        let response1 = rx3.recv().unwrap();
        assert_eq!(
            response1,
            "*2\r\n$6\r\nvalue3\r\n$6\r\nvalue2\r\n".to_string()
        );

        let (tx4, rx4): (Sender<String>, Receiver<String>) = mpsc::channel();
        let buffer_vec_mock = vec!["llen".to_string(), "value".to_string()];
        tx1.send((buffer_vec_mock, tx4)).unwrap();

        let response1 = rx4.recv().unwrap();
        assert_eq!(response1, ":0\r\n".to_string());
    }
}
