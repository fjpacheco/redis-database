use crate::commands::{
    keys::{
        _type::Type, clean::Clean, copy::Copy, del::Del, exists::Exists, expire::Expire,
        expireat::ExpireAt, key_command::Keys, persist::Persist, rename::Rename, sort::Sort,
        touch::Touch, ttl::Ttl,
    },
    lists::{
        lindex::LIndex, llen::Llen, lpop::LPop, lpush::LPush, lpushx::LPushx, lrange::Lrange,
        lrem::Lrem, lset::Lset, rpop::RPop, rpush::RPush, rpushx::RPushx,
    },
    pubsub::{
        publish::Publish, pubsub_command::Pubsub, subscribe_cf::SubscribeCF,
        subscribe_cl::SubscribeCL, unsubscribe_cf::UnsubscribeCF, unsubscribe_cl::UnsubscribeCL,
    },
    server::{
        config::Config, dbsize::Dbsize, flushdb::FlushDB, info_db::InfoDB, info_sv::InfoSV,
        monitor::Monitor, notify_monitors::NotifyMonitors, shutdown::Shutdown,
    },
    sets::{sadd::Sadd, scard::Scard, sismember::Sismember, smembers::Smembers, srem::Srem},
    strings::{
        append::Append, decrby::Decrby, get::Get, getdel::Getdel, getset::Getset, incrby::Incrby,
        mget::Mget, mset::Mset, set::Set, strlen::Strlen,
    },
};

use crate::tcp_protocol::client_list::ClientList;
use crate::tcp_protocol::BoxedCommand;
use crate::Database;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use super::client_atributes::client_fields::ClientFields;
use crate::tcp_protocol::server_redis_atributes::ServerRedisAtributes;

pub struct RunnablesMap<T> {
    elements: HashMap<String, Arc<BoxedCommand<T>>>,
}
#[macro_export]
macro_rules! get_runnables {
    ( $map:expr, $( $x:ident ),* ) => {
        {
            $(
                $map.insert(stringify!($x).to_string().to_lowercase(), Arc::new(Box::new($x)));
            )*
            $map
        }
    };
}

impl<T> RunnablesMap<T> {
    pub fn new(map: HashMap<String, Arc<BoxedCommand<T>>>) -> Self {
        Self { elements: map }
    }

    pub fn get(&self, string: &str) -> Option<Arc<BoxedCommand<T>>> {
        self.elements
            .get(string)
            .map(|summoner| Arc::clone(summoner))
    }

    pub fn contains_key(&self, string: &str) -> bool {
        self.elements.contains_key(string)
    }

    pub fn database() -> RunnablesMap<Arc<Mutex<Database>>> {
        let mut map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<Database>>>>> = HashMap::new();
        map = get_runnables!(
            map, Type, Clean, Copy, Del, Exists, Expire, ExpireAt, Keys, Persist, Rename, Sort,
            Touch, Ttl, LIndex, Llen, LPop, LPush, LPushx, Lrange, Lrem, Lset, RPop, RPush, RPushx,
            Dbsize, FlushDB, Sadd, Scard, Sismember, Smembers, Srem, Append, Decrby, Get, Getdel,
            Getset, Incrby, Mget, Mset, Set, Strlen
        );
        map.insert(
            "info".to_string().to_lowercase(),
            Arc::new(Box::new(InfoDB)),
        );
        RunnablesMap { elements: map }
    }

    pub fn server() -> RunnablesMap<ServerRedisAtributes> {
        let mut map: HashMap<String, Arc<BoxedCommand<ServerRedisAtributes>>> = HashMap::new();

        /*map.insert(
            String::from("shutdown"),
            Arc::new(Box::new(server::shutdown::Shutdown)),
        );
        map.insert(
            String::from("config get"),
            Arc::new(Box::new(server::config_get::ConfigGet)),
        );
        map.insert(
            String::from("notifymonitors"),
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

        RunnablesMap { elements: map }*/
        map = get_runnables!(map, Publish, Pubsub, Config, NotifyMonitors, Shutdown);
        map.insert(
            "subscribe".to_string().to_lowercase(),
            Arc::new(Box::new(SubscribeCL)),
        );
        map.insert(
            "unsubscribe".to_string().to_lowercase(),
            Arc::new(Box::new(UnsubscribeCL)),
        );
        map.insert(
            "info".to_string().to_lowercase(),
            Arc::new(Box::new(InfoSV)),
        );
        RunnablesMap { elements: map }
    }

    pub fn client_list() -> RunnablesMap<Arc<Mutex<ClientList>>> {
        let map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientList>>>>> = HashMap::new();
        RunnablesMap { elements: map }
    }

    // Hace falta agregarle metodos de ejecutor
    pub fn executor() -> RunnablesMap<Arc<Mutex<ClientFields>>> {
        let mut map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>> = HashMap::new();
        map.insert(String::from("monitor"), Arc::new(Box::new(Monitor)));
        map.insert(String::from("subscribe"), Arc::new(Box::new(SubscribeCF)));
        map.insert(
            String::from("unsubscribe"),
            Arc::new(Box::new(UnsubscribeCF)),
        );
        RunnablesMap { elements: map }
    }

    pub fn subscriber() -> RunnablesMap<Arc<Mutex<ClientFields>>> {
        let mut map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>> = HashMap::new();
        map.insert(String::from("subscribe"), Arc::new(Box::new(SubscribeCF)));
        map.insert(
            String::from("unsubscribe"),
            Arc::new(Box::new(UnsubscribeCF)),
        );
        RunnablesMap { elements: map }
    }
}
