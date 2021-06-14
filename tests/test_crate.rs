#[cfg(test)]
mod testings_redis {
    extern crate redis;
    use std::thread;

    use redis::RedisError;
    use redis_rust::{redis_config::RedisConfig, tcp_protocol::server::ServerRedis};

    #[test]
    fn test01_crate() {
        let _server_handler = thread::spawn(|| ServerRedis::start(vec![]));

        let client = redis::Client::open(
            "redis://".to_owned()
                + &RedisConfig::default().ip()
                + ":"
                + &RedisConfig::default().port()
                + "/",
        )
        .unwrap();
        let mut conection_client = client.get_connection().unwrap();

        let received_3: Result<String, RedisError> = redis::cmd("set")
            .arg("Agustín")
            .arg("Firmapaz")
            .query(&mut conection_client);
        println!("set Agustín Firmapaz => {:?}", received_3);
        let received_3: Result<String, RedisError> = redis::cmd("set")
            .arg("Martina")
            .arg("Panetta")
            .query(&mut conection_client);
        println!("set Martina Panetta => {:?}", received_3);
        let received_54: Result<String, RedisError> = redis::cmd(" ").query(&mut conection_client);
        println!("5 => {:?}", received_54);

        let received_4: Result<String, RedisError> = redis::cmd("get")
            .arg("Martina")
            .query(&mut conection_client);
        println!("get Martina => {:?}", received_4);
        let received_4: Result<String, RedisError> = redis::cmd("get")
            .arg("Agustín")
            .query(&mut conection_client);
        println!("get Agustín => {:?}", received_4);
    }
}
