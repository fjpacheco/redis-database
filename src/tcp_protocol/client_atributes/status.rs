use std::collections::HashSet;
use std::mem;

use crate::tcp_protocol::runnables_map::RunnablesMap;

pub enum Status {
    Executor,
    Subscriber,
    Monitor,
    Dead,
}

impl Status {
    pub fn replace(&mut self, new_status: Status) -> Status {
        mem::replace(self, new_status)
    }

    pub fn update_map(&self) -> Option<RunnablesMap<Status>> {
        match self {
            Self::Executor => Some(RunnablesMap::<Status>::executor()),
            Self::Subscriber => Some(RunnablesMap::<Status>::subscriber()),
            _ => None,
        }
    }

}
