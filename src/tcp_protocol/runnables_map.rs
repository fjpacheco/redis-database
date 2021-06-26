use std::collections::HashMap;

use crate::commands::*;
use crate::tcp_protocol::client_atributes::status::Status;
use crate::{commands::Runnable, Database};

use super::server::ServerRedis;

pub struct RunnablesMap<T> {
    elements: HashMap<String, Box<dyn Runnable<T> + Send + Sync>>,
}

impl<T> RunnablesMap<T> {
    pub fn new(map: HashMap<String, Box<dyn Runnable<T> + Send + Sync>>) -> Self {
        Self { elements: map }
    }
    // CHECK
    #[allow(clippy::borrowed_box)]
    pub fn get(&self, string: &str) -> Option<&Box<dyn Runnable<T> + Send + Sync>> {
        self.elements.get(string)
    }

    pub fn database() -> RunnablesMap<Database> {
        let mut map: HashMap<String, Box<dyn Runnable<Database> + Send + Sync>> = HashMap::new();
        map.insert(String::from("set"), Box::new(strings::set::Set));
        map.insert(String::from("get"), Box::new(strings::get::Get));
        map.insert(String::from("strlen"), Box::new(strings::strlen::Strlen));

        RunnablesMap { elements: map }
    }

    pub fn server() -> RunnablesMap<ServerRedis> {
        let mut map: HashMap<String, Box<dyn Runnable<ServerRedis> + Send + Sync>> = HashMap::new();
        map.insert(
            String::from("shutdown"),
            Box::new(server::shutdown::Shutdown),
        );
        map.insert(
            String::from("config set"),
            Box::new(server::config_set::ConfigSet),
        );

        RunnablesMap { elements: map }
    }

    // Hace falta agregarle metodos de ejecutor
    pub fn executor() -> RunnablesMap<Status> {
        let map: HashMap<String, Box<dyn Runnable<Status> + Send + Sync>> = HashMap::new();
        RunnablesMap { elements: map }
    }

    // Hace falta agregarle metodos de suscriptor
    pub fn subscriber() -> RunnablesMap<Status> {
        let map: HashMap<String, Box<dyn Runnable<Status> + Send + Sync>> = HashMap::new();
        RunnablesMap { elements: map }
    }
}
