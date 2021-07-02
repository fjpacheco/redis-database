use std::{
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use redis::RedisError;
use redis_rust::{
    native_types::ErrorStruct, redis_config::RedisConfig, tcp_protocol::server::ServerRedis,
};

pub struct TestContext {
    client: redis::Client,
}

impl TestContext {
    pub fn new() -> Result<Self, ErrorStruct> {
        let _server_thread: JoinHandle<Result<(), ErrorStruct>> = spawn(|| {
            ServerRedis::start(vec![])?;
            Ok(())
        });

        let client = redis::Client::open(
            "redis://".to_owned()
                + &RedisConfig::default().ip()
                + ":"
                + &RedisConfig::default().port()
                + "/",
        )
        .unwrap();

        let mut _con;

        let millisecond = Duration::from_millis(1);
        let mut retries = 0;
        loop {
            match client.get_connection() {
                Err(err) => {
                    if err.is_connection_refusal() {
                        sleep(millisecond);
                        retries += 1;
                        if retries > 100000 {
                            panic!("Tried to connect too many times, last error: {}", err);
                        }
                    } else {
                        panic!("Could not connect: {}", err);
                    }
                }
                Ok(x) => {
                    _con = x;
                    break;
                }
            }
        }

        //TODO: Ejecutar esto always!
        //redis::cmd("FLUSHDB").execute(&mut _con);

        Ok(Self { client })
    }

    pub fn connection(&self) -> redis::Connection {
        self.client.get_connection().unwrap()
    }

    pub fn server_off(&mut self) {
        let _: Result<(), RedisError> = redis::cmd("shutdown").query(&mut self.client);
    }
}
