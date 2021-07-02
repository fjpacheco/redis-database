use std::collections::HashMap;

use crate::commands::*;
use crate::{commands::Runnable, Database};

use super::client_atributes::status::Status;
use super::server::ServerRedisAtributes;

pub struct RunnablesMap<T> {
    elements: HashMap<String, Box<dyn Runnable<T> + Send + Sync>>,
}

impl<T> RunnablesMap<T> {
    pub fn new(map: HashMap<String, Box<dyn Runnable<T> + Send + Sync>>) -> Self {
        Self { elements: map }
    }

    pub fn get(&self, string: &str) -> Option<&(dyn Runnable<T> + Send + Sync)> {
        self.elements.get(string).map(|x| x.as_ref())
    }

    pub fn database() -> RunnablesMap<Database> {
        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("set"), Box::new(strings::set::Set));
        map.insert(String::from("get"), Box::new(strings::get::Get));
        map.insert(String::from("strlen"), Box::new(strings::strlen::Strlen));

        RunnablesMap { elements: map }
    }

    pub fn server() -> RunnablesMap<ServerRedisAtributes> {
        let mut map: HashMap<String, Box<dyn Runnable<ServerRedisAtributes> + Send + Sync>> =
            HashMap::new();
        map.insert(
            String::from("shutdown"),
            Box::new(server::shutdown::Shutdown),
        );
        map.insert(
            String::from("config get"),
            Box::new(server::config_get::ConfigGet),
        );
        map.insert(
            String::from("notify_monitors"),
            Box::new(server::notify_monitors::NotifyMonitors),
        );
        map.insert(
            String::from("clear_client"),
            Box::new(server::clear_client::ClearClient),
        );

        RunnablesMap { elements: map }
    }

    // Hace falta agregarle metodos de ejecutor
    pub fn executor() -> RunnablesMap<Status> {
        let mut map: HashMap<String, Box<dyn Runnable<Status> + Send + Sync>> = HashMap::new();
        map.insert(String::from("monitor"), Box::new(server::monitor::Monitor));
        //map.insert(String::from("subscribe"), Box::new(pubsub::subscribe::Subscribe));
        //map.insert(String::from("unsubscribe"), Box::new(pubsub::unsubscribe::Unsubscribe));
        RunnablesMap { elements: map }
    }

    // Hace falta agregarle metodos de suscriptor
    pub fn subscriber() -> RunnablesMap<Status> {
        let map: HashMap<String, Box<dyn Runnable<Status> + Send + Sync>> = HashMap::new();
        //map.insert(String::from("subscribe"), Box::new(pubsub::subscribe::Subscribe));
        //map.insert(String::from("unsubscribe"), Box::new(pubsub::unsubscribe::Unsubscribe));
        RunnablesMap { elements: map }
    }
}
