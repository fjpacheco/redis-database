use std::collections::HashSet;
use crate::messages::redis_messages::not_valid_monitor;
use crate::messages::redis_messages::not_valid_pubsub;
use crate::native_types::ErrorStruct;
use crate::tcp_protocol::client_atributes::{status::Status, status_answer::StatusAnswer};
use crate::tcp_protocol::runnables_map::RunnablesMap;
use crate::messages::redis_messages::unexpected_behaviour;

use std::net::SocketAddrV4;

pub struct ClientStatus {
    map: Option<RunnablesMap<Status>>,
    status: Status,
    subscriptions: HashSet<String>,
    address: SocketAddrV4,
}

impl ClientStatus {
    pub fn new(address: SocketAddrV4) -> ClientStatus {
        ClientStatus {
            map: Some(RunnablesMap::<Status>::executor()),
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

    pub fn review_command(&mut self, command: Vec<String>) -> StatusAnswer {
        match self.status {
            Status::Executor => self.rc_case_executor(command),
            Status::Subscriber => self.rc_case_subscriber(command),
            Status::Monitor => StatusAnswer::Break(ErrorStruct::new(
                not_valid_monitor().get_prefix(),
                not_valid_monitor().get_message(),
            )),
            Status::Dead => panic!(),
        }
    }

    fn rc_case_subscriber(&mut self, mut command: Vec<String>) -> StatusAnswer {
        if let Some(runnable) = self.map.as_ref().unwrap().get(command.get(0).unwrap()) {
            command.remove(0);
            let response = runnable.run(command, &mut self.status);
            self.update_map();
            StatusAnswer::Done(response)
        } else {
            StatusAnswer::Break(ErrorStruct::new(
                not_valid_pubsub().get_prefix(),
                not_valid_pubsub().get_message(),
            ))
        }
    }

    fn rc_case_executor(&mut self, mut command: Vec<String>) -> StatusAnswer {
        if let Some(runnable) = self.map.as_ref().unwrap().get(command.get(0).unwrap()) {
            command.remove(0);
            let response = runnable.run(command, &mut self.status);
            self.update_map();
            StatusAnswer::Done(response)
        } else {
            StatusAnswer::Continue(command)
        }
    }

    fn update_map(&mut self) {
        if let Some(new_map) = self.status.update_map() {
            self.map.replace(new_map);
        } else {
            self.map.take();
        }
    }

    pub fn add_subscriptions(&mut self, channels: &Vec<String>) -> Result<isize, ErrorStruct>{
        match self.status {
            Status::Executor => self.as_case_executor(channels),
            Status::Subscriber => self.as_case_subscriber(channels),
            _ => Err(ErrorStruct::new(
                unexpected_behaviour("Dead client (or monitor) is trying to execute invalid command").get_prefix(),
                unexpected_behaviour("Dead client (or monitor) is trying to execute invalid command").get_message(),
            )),
        }
    }

    fn as_case_executor(&mut self, channels: &Vec<String>) -> Result<isize, ErrorStruct> {
        let added = self.add_channels(channels);
        self.replace_status(Status::Subscriber);
        Ok(added)
    }

    fn as_case_subscriber(&mut self, channels: &Vec<String>) -> Result<isize, ErrorStruct> {
        let added = self.add_channels(channels);
        Ok(added)
    }

    pub fn remove_subscriptions(&mut self, channels: &Vec<String>) -> Result<isize, ErrorStruct>{
        match &self.status {
            Status::Executor => Ok(0),
            Status::Subscriber => self.rs_case_subscriber(channels),
            _ => Err(ErrorStruct::new(
                unexpected_behaviour("Dead client (or monitor) is trying to execute invalid command").get_prefix(),
                unexpected_behaviour("Dead client (or monitor) is trying to execute invalid command").get_message(),
            )),
        }
    }

    fn rs_case_subscriber(&mut self, channels: &Vec<String>) -> Result<isize, ErrorStruct> {
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

    fn add_channels(&mut self, new_channels: &Vec<String>) -> isize {
        for channel in new_channels.iter() {
            self.subscriptions.insert(String::from(channel));
        }
        new_channels.len() as isize
    }
    
    fn remove_channels(&mut self, new_channels: &Vec<String>) -> isize {
        for channel in new_channels.iter() {
            self.subscriptions.remove(channel);
        }
        new_channels.len() as isize
    }
}



/*#[cfg(test)]

mod test_client_status {

    use super::*;
    use std::net::Ipv4Addr;

    fn test01_add_subscriptions(){

        let status = ClientStatus::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        let added = status.add_subscriptions(vec!["telefe".to_string(), "trece".to_string()]);

    }
}*/
