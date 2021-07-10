use crate::commands::*;
use crate::tcp_protocol::client_list::ClientList;
use crate::tcp_protocol::BoxedCommand;
use crate::Database;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

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
        /*if let Some(summoner) = self.elements.get(string) {
            Some(Arc::clone(summoner))
        } else {
            None
        }*/
        self.elements
            .get(string)
            .map(|summoner| Arc::clone(summoner))
    }

    pub fn contains_key(&self, string: &str) -> bool {
        self.elements.contains_key(string)
    }

    pub fn database() -> RunnablesMap<Database> {
        let mut map: HashMap<String, Arc<BoxedCommand<Database>>> = HashMap::new();
        map.insert(String::from("set"), Arc::new(Box::new(strings::set::Set)));
        map.insert(String::from("get"), Arc::new(Box::new(strings::get::Get)));
        map.insert(
            String::from("strlen"),
            Arc::new(Box::new(strings::strlen::Strlen)),
        );

        RunnablesMap { elements: map }
    }

    pub fn server() -> RunnablesMap<ServerRedisAtributes> {
        let mut map: HashMap<String, Arc<BoxedCommand<ServerRedisAtributes>>> = HashMap::new();
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
            String::from("subscribe"),
            Arc::new(Box::new(pubsub::subscribe_cl::SubscribeCL)),
        );
        map.insert(
            String::from("unsubscribe"),
            Arc::new(Box::new(pubsub::unsubscribe_cl::UnsubscribeCL)),
        );
        map.insert(
            String::from("publish"),
            Arc::new(Box::new(pubsub::publish::Publish)),
        );

        RunnablesMap { elements: map }
    }

    pub fn client_list() -> RunnablesMap<Arc<Mutex<ClientList>>> {
        let map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientList>>>>> = HashMap::new();
        //map.insert(String::from("subscribe"), Arc::new(Box::new(pubsub::subscribe_cl::SubscribeCL)));
        //map.insert(String::from("unsubscribe"), Arc::new(Box::new(pubsub::unsubscribe_cl::UnsubscribeCL)));
        RunnablesMap { elements: map }
    }

    // Hace falta agregarle metodos de ejecutor
    pub fn executor() -> RunnablesMap<Arc<Mutex<ClientFields>>> {
        let mut map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>> = HashMap::new();
        map.insert(
            String::from("monitor"),
            Arc::new(Box::new(server::monitor::Monitor)),
        );
        map.insert(
            String::from("subscribe"),
            Arc::new(Box::new(pubsub::subscribe_cf::SubscribeCF)),
        );
        map.insert(
            String::from("unsubscribe"),
            Arc::new(Box::new(pubsub::unsubscribe_cf::UnsubscribeCF)),
        );
        RunnablesMap { elements: map }
    }

    pub fn subscriber() -> RunnablesMap<Arc<Mutex<ClientFields>>> {
        let mut map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>> = HashMap::new();
        map.insert(
            String::from("subscribe"),
            Arc::new(Box::new(pubsub::subscribe_cf::SubscribeCF)),
        );
        map.insert(
            String::from("unsubscribe"),
            Arc::new(Box::new(pubsub::unsubscribe_cf::UnsubscribeCF)),
        );
        RunnablesMap { elements: map }
    }
}
