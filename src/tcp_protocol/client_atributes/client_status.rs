use crate::messages::redis_messages::not_valid_monitor;
use crate::messages::redis_messages::not_valid_pubsub;
use crate::native_types::ErrorStruct;
use crate::tcp_protocol::client_atributes::{status::Status, status_answer::StatusAnswer};
use crate::tcp_protocol::runnables_map::RunnablesMap;

pub struct ClientStatus {
    map: Option<RunnablesMap<Status>>,
    status: Status,
}

impl ClientStatus {
    pub fn new() -> ClientStatus {
        ClientStatus {
            map: Some(RunnablesMap::<Status>::executor()),
            status: Status::Executor,
        }
    }

    pub fn review_command(&mut self, command: Vec<String>) -> StatusAnswer {
        match self.status {
            Status::Executor => self.fun_executor(command),
            Status::Subscriber(_) => self.fun_subscriber(command),
            Status::Monitor => StatusAnswer::Break(ErrorStruct::new(
                not_valid_monitor().get_prefix(),
                not_valid_monitor().get_message(),
            )),
            Status::Dead => panic!(),
        }
    }

    fn fun_subscriber(&mut self, mut command: Vec<String>) -> StatusAnswer {
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

    fn fun_executor(&mut self, mut command: Vec<String>) -> StatusAnswer {
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
}

#[cfg(test)]

mod test_client_status {

    //use super::*;

    //   fn test01_
}
