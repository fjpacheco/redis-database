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
        publish::Publish, pubsub_command::Pubsub, subscribe_cf::SubscribeCf,
        subscribe_cl::SubscribeCl, unsubscribe_cf::UnsubscribeCf, unsubscribe_cl::UnsubscribeCl,
    },
    server::{
        config::Config, dbsize::Dbsize, flushdb::FlushDb, info_db::InfoDb, info_sv::InfoSv,
        monitor::Monitor, notify_monitors::NotifyMonitors, save::Save, shutdown::Shutdown,
    },
    sets::{sadd::Sadd, scard::Scard, sismember::Sismember, smembers::Smembers, srem::Srem},
    strings::{
        append::Append, decrby::Decrby, get::Get, getdel::Getdel, getset::Getset, incrby::Incrby,
        mget::Mget, mset::Mset, set::Set, strlen::Strlen,
    },
};

use crate::tcp_protocol::BoxedCommand;
use crate::Database;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use super::client_atributes::client_fields::ClientFields;
use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;

/// Associate a command's name with a runnable.
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
    /// Creats an empty instance of the runnables map.
    pub fn new(map: HashMap<String, Arc<BoxedCommand<T>>>) -> Self {
        Self { elements: map }
    }

    /// Returns the runnable associated with the given command's name.
    pub fn get(&self, string: &str) -> Option<Arc<BoxedCommand<T>>> {
        self.elements
            .get(string)
            .map(|summoner| Arc::clone(summoner))
    }

    /// Indicates if the map contains the given command's name.
    ///
    /// # Return value
    /// [bool]: true if the map contains the key.
    pub fn contains_key(&self, string: &str) -> bool {
        self.elements.contains_key(string)
    }

    /// Creates a default instance with database runnables.
    pub fn database() -> RunnablesMap<Arc<Mutex<Database>>> {
        let mut map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<Database>>>>> = HashMap::new();
        map = get_runnables!(
            map, Type, Clean, Copy, Del, Exists, Expire, ExpireAt, Keys, Persist, Rename, Sort,
            Touch, Ttl, LIndex, Llen, LPop, LPush, LPushx, Lrange, Lrem, Lset, RPop, RPush, RPushx,
            Dbsize, FlushDb, Sadd, Scard, Sismember, Smembers, Srem, Append, Decrby, Get, Getdel,
            Getset, Incrby, Mget, Mset, Set, Strlen, Save
        );
        map.insert(
            "info".to_string().to_lowercase(),
            Arc::new(Box::new(InfoDb)),
        );
        RunnablesMap { elements: map }
    }

    /// Creates a default instance with server runnables.
    pub fn server() -> RunnablesMap<ServerRedisAttributes> {
        let mut map: HashMap<String, Arc<BoxedCommand<ServerRedisAttributes>>> = HashMap::new();

        map = get_runnables!(map, Publish, Pubsub, Config, NotifyMonitors, Shutdown);
        map.insert(
            "subscribe".to_string().to_lowercase(),
            Arc::new(Box::new(SubscribeCl)),
        );
        map.insert(
            "unsubscribe".to_string().to_lowercase(),
            Arc::new(Box::new(UnsubscribeCl)),
        );
        map.insert(
            "info".to_string().to_lowercase(),
            Arc::new(Box::new(InfoSv)),
        );
        RunnablesMap { elements: map }
    }

    /// Creates a default instance with Executor runnables.
    pub fn executor() -> RunnablesMap<Arc<Mutex<ClientFields>>> {
        let mut map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>> = HashMap::new();
        map.insert(String::from("monitor"), Arc::new(Box::new(Monitor)));
        map.insert(String::from("subscribe"), Arc::new(Box::new(SubscribeCf)));
        map.insert(
            String::from("unsubscribe"),
            Arc::new(Box::new(UnsubscribeCf)),
        );
        RunnablesMap { elements: map }
    }

    /// Creates a default instance with subscriber runnables.
    pub fn subscriber() -> RunnablesMap<Arc<Mutex<ClientFields>>> {
        let mut map: HashMap<String, Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>> = HashMap::new();
        map.insert(String::from("subscribe"), Arc::new(Box::new(SubscribeCf)));
        map.insert(
            String::from("unsubscribe"),
            Arc::new(Box::new(UnsubscribeCf)),
        );
        RunnablesMap { elements: map }
    }
}
