use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use crate::tcp_protocol::BoxedCommand;
use crate::commands::*;
use crate::{commands::Runnable, Database};

use super::client_atributes::status::Status;
use super::client_atributes::client_fields::ClientFields;
use super::server::ServerRedisAtributes;

pub struct RunnablesMap<T> {
    elements: HashMap<String, Arc<BoxedCommand<T>>>,
}

impl<T> RunnablesMap<T> {
    pub fn new(map: HashMap<String, Arc<BoxedCommand<T>>>) -> Self {
        Self { elements: map }
    }
    
    pub fn get(&self, string: &str) -> Option<Arc<BoxedCommand<T>>> {
        if let Some(summoner) = self.elements.get(string) {
            Some(Arc::clone(summoner))
        } else {
            None
        }
    }

    pub fn contains_key(&self, string: &str) -> bool {
        self.elements.contains_key(string)
    }

    pub fn database() -> RunnablesMap<Database> {
        let mut map: HashMap<String, Arc<BoxedCommand<Database>>> = HashMap::new();
        map.insert(String::from("set"), Arc::new(Box::new(strings::set::Set)));
        map.insert(String::from("get"), Arc::new(Box::new(strings::get::Get)));
        map.insert(String::from("strlen"), Arc::new(Box::new(strings::strlen::Strlen)));

        RunnablesMap { elements: map }
    }


    pub fn server() -> RunnablesMap<ServerRedisAtributes> {
        let mut map: HashMap<String, Arc<BoxedCommand<ServerRedisAtributes>>> =
            HashMap::new();
        map.insert(
            String::from("shutdown"),
            Arc::new(Box::new(server::shutdown::Shutdown)),
        );
        map.insert(
            String::from("config get"),
            Arc::new(Box::new(server::config_get::ConfigGet)),
        );
        map.insert(
            String::from("notify_monitors"),
            Arc::new(Box::new(server::notify_monitors::NotifyMonitors)),
        );
        map.insert(
            String::from("clear_client"),
            Arc::new(Box::new(server::clear_client::ClearClient)),
        );

        RunnablesMap { elements: map }
    }

    // Hace falta agregarle metodos de ejecutor
    pub fn executor() -> RunnablesMap<Arc<Mutex<ClientFields>>> {
        let map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>> = HashMap::new();
        //map.insert(String::from("monitor"), Box::new(server::monitor::Monitor));
        //map.insert(String::from("subscribe"), Box::new(pubsub::subscribe::Subscribe));
        //map.insert(String::from("unsubscribe"), Box::new(pubsub::unsubscribe::Unsubscribe));
        RunnablesMap { elements: map }
    }

    pub fn subscriber() -> RunnablesMap<Arc<Mutex<ClientFields>>> {
        let map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>> = HashMap::new();
        //map.insert(String::from("subscribe"), Box::new(pubsub::subscribe::Subscribe));
        //map.insert(String::from("unsubscribe"), Box::new(pubsub::unsubscribe::Unsubscribe));
        RunnablesMap { elements: map }
    }
}
