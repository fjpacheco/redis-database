use crate::messages::redis_messages::not_valid_monitor;
use crate::messages::redis_messages::not_valid_pubsub;
use crate::messages::redis_messages::unexpected_behaviour;
use crate::native_types::ErrorStruct;
use crate::tcp_protocol::client_atributes::{status::Status, status_answer::StatusAnswer};
use crate::tcp_protocol::runnables_map::RunnablesMap;
use std::sync::Mutex;
use crate::tcp_protocol::BoxedCommand;
use std::sync::Arc;
use std::net::Ipv4Addr;
use std::collections::HashSet;

use std::net::SocketAddrV4;

pub struct ClientFields {
    map: Option<RunnablesMap<Arc<Mutex<ClientFields>>>>,
    status: Status,
    subscriptions: HashSet<String>,
    pub address: SocketAddrV4,
}

impl ClientFields {
    pub fn new(address: SocketAddrV4) -> ClientFields {
        ClientFields {
            map: Some(RunnablesMap::<Arc<Mutex<ClientFields>>>::executor()),
            status: Status::Executor,
            subscriptions: HashSet::new(),
            address,
        }
    }

    pub fn replace_status(&mut self, new_status: Status) -> Status {
        let old_status = self.status.replace(new_status);
        self.update_map();
        old_status
    }

    pub fn status(&self) -> Option<&Status> {
        Some(&self.status)
    }

    pub fn is_subscripted_to(&self, channel: &str) -> bool {
        self.subscriptions.contains(channel)
    }

    pub fn is_allowed_to(&self, command: &str) -> Result<(), ErrorStruct> {
        match self.status {
            Status::Executor => Ok(()),
            Status::Subscriber => {
                if self.map.as_ref().unwrap().contains_key(command) {
                    Ok(())
                } else {
                    Err(ErrorStruct::new(
                        not_valid_pubsub().get_prefix(),
                        not_valid_pubsub().get_message(),
                    ))
                }
            },
            _ => Err(ErrorStruct::new(
                not_valid_monitor().get_prefix(),
                not_valid_monitor().get_message(),
                )),
        }
    }

    pub fn review_command(&mut self, command: &Vec<String>) -> Result<Option<Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>>, ErrorStruct> {
        match self.status {
            Status::Executor => self.rc_case_executor(command),
            Status::Subscriber => self.rc_case_subscriber(command),
            Status::Monitor => Err(ErrorStruct::new(
                not_valid_monitor().get_prefix(),
                not_valid_monitor().get_message(),
            )),
            Status::Dead => panic!(),
        }
    }

    pub fn is_monitor_notifiable(&self) -> bool {
        self.status == Status::Monitor
    }

    fn rc_case_subscriber(&mut self, command: &Vec<String>) -> Result<Option<Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>>, ErrorStruct> {
        if let Some(runnable) = self.map.as_ref().unwrap().get(command.get(0).unwrap()) {
            Ok(Some(runnable))
        } else {
            Err(ErrorStruct::new(
                not_valid_pubsub().get_prefix(),
                not_valid_pubsub().get_message(),
            ))
        }
    }

    fn rc_case_executor(&mut self, command: &Vec<String>) -> Result<Option<Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>>, ErrorStruct> {
        Ok(self.map.as_ref().unwrap().get(command.get(0).unwrap()))
    }

    fn update_map(&mut self) {
        if let Some(new_map) = self.status.update_map() {
            self.map.replace(new_map);
        } else {
            self.map.take();
        }
    }

    pub fn add_subscriptions(&mut self, channels: &[String]) -> Result<isize, ErrorStruct> {
        match self.status {
            Status::Executor => self.as_case_executor(channels),
            Status::Subscriber => self.as_case_subscriber(channels),
            _ => Err(ErrorStruct::new(
                unexpected_behaviour(
                    "Dead client (or monitor) is trying to execute invalid command",
                )
                .get_prefix(),
                unexpected_behaviour(
                    "Dead client (or monitor) is trying to execute invalid command",
                )
                .get_message(),
            )),
        }
    }

    fn as_case_executor(&mut self, channels: &[String]) -> Result<isize, ErrorStruct> {
        let added = self.add_channels(channels);
        self.replace_status(Status::Subscriber);
        Ok(added)
    }

    fn as_case_subscriber(&mut self, channels: &[String]) -> Result<isize, ErrorStruct> {
        let added = self.add_channels(channels);
        Ok(added)
    }

    pub fn remove_subscriptions(&mut self, channels: &[String]) -> Result<isize, ErrorStruct> {
        match &self.status {
            Status::Executor => Ok(0),
            Status::Subscriber => self.rs_case_subscriber(channels),
            _ => Err(ErrorStruct::new(
                unexpected_behaviour(
                    "Dead client (or monitor) is trying to execute invalid command",
                )
                .get_prefix(),
                unexpected_behaviour(
                    "Dead client (or monitor) is trying to execute invalid command",
                )
                .get_message(),
            )),
        }
    }

    fn rs_case_subscriber(&mut self, channels: &[String]) -> Result<isize, ErrorStruct> {
        if channels.is_empty() {
            self.status.replace(Status::Executor);
            Ok(0)
        } else {
            let actual_size = self.remove_channels(channels);
            if actual_size == 0 {
                self.replace_status(Status::Executor);
            }
            Ok(actual_size)
        }
    }

    fn add_channels(&mut self, new_channels: &[String]) -> isize {
        for channel in new_channels.iter() {
            self.subscriptions.insert(String::from(channel));
        }
        new_channels.len() as isize
    }

    fn remove_channels(&mut self, new_channels: &[String]) -> isize {
        for channel in new_channels.iter() {
            self.subscriptions.remove(channel);
        }
        new_channels.len() as isize
    }
}

impl Default for ClientFields {
    fn default() -> ClientFields {
        ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(1, 0, 0, 127), 8080))
    }
}

#[cfg(test)]
mod test_client_status {

    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test01_initial_state(){

        let status = ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        assert_eq!(status.status(), Some(&Status::Executor));

    }

    #[test]
    fn test02_add_subscriptions(){

        let mut status = ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        let added = status.add_subscriptions(&mut vec!["telefe".to_string(), "trece".to_string()]);
        assert_eq!(added.unwrap(), 2);
        assert_eq!(status.status(), Some(&Status::Subscriber));

    }
}
