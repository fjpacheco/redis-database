use crate::messages::redis_messages::not_valid_pubsub;
use crate::native_types::ErrorStruct;
use crate::commands::Runnable;
use crate::tcp_protocol::runnables_map::RunnablesMap;
use std::collections::HashSet;
use std::mem;


pub enum Status{
    Executer,
    Subscriber(HashSet<String>),
    Monitor,
    Dead,
}

impl Status {

    pub fn replace(&mut self, new_status: Status) -> Status{
        mem::replace(self, new_status)
    }

}

pub enum StatusAnswer {
    Continue(Vec<String>),
    Done(Result<String, ErrorStruct>),
    Break(Option<ErrorStruct>),
}

pub struct ClientStatus {

    map: RunnablesMap<Status>,
    status: Status,

}

impl ClientStatus {
    
    pub fn review_command(&mut self, mut command: Vec<String>) -> StatusAnswer {
        match self.status {
            Status::Executer => {
                if let Some(runnable) = self.map.get(command.get(0).unwrap()) {
                    command.remove(0);
                    StatusAnswer::Done(runnable.run(command, &mut self.status))
                } else {
                    StatusAnswer::Continue(command)
                }
            },
            Status::Subscriber(_) => {
                if let Some(runnable) = self.map.get(command.get(0).unwrap()) {
                    command.remove(0);
                    StatusAnswer::Done(runnable.run(command, &mut self.status))
                } else {
                    StatusAnswer::Break(Some(ErrorStruct::new(
                        not_valid_pubsub().get_prefix(),
                        not_valid_pubsub().get_message(),
                    )))
                }
            },
            Status::Monitor => StatusAnswer::Break(None),
            Status::Dead => panic!(),
        }
    }

    /*fn search_runnable(map: &'static RunnablesMap<ClientStatus>, command: &str) -> StatusAnswer{
        if let Some(runnable) = map.get(command) {
            StatusAnswer::Execute(Some(runnable))
        } else {
            StatusAnswer::Continue
        }
    }

    fn valid_runnable(map: &'static RunnablesMap<ClientStatus>, command: &str) -> StatusAnswer {
        if let Some(runnable) = map.get(command) {
            StatusAnswer::Execute(Some(runnable))
        } else {
            StatusAnswer::Break(Some(ErrorStruct::new(
                not_valid_pubsub().get_prefix(),
                not_valid_pubsub().get_message(),
            )))
        }
    }*/
}

//fn process_executer(client_status: &mut ClientStatus, command: &str) -> 



