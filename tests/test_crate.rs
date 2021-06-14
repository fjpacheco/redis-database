#[cfg(test)]
mod testings_redis {
    extern crate redis;
    use std::thread;

    use redis::RedisError;
    use redis_rust::{redis_config::RedisConfig, tcp_protocol::server::ServerRedis};

    #[test]
    fn test01_crate() {
        let _server_handler = thread::spawn(move || {
            let _server = ServerRedis::start(vec![]);
        });

        // Ejecutar en consola antes del test => cargo run --bin server -- 6379
        // TODO: invstigar como poder levantar el server desde un test... es un doloooooooor tener que levantarlo manualmente
        // TODO: investigar la organizacion de modulos! creo q ahi esta la clave => deberiamos re-ajustar all modulos
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
        /*
        for _ in 0..1000{
            for i in 0..9{
                let received_3: Result<String, RedisError> = redis::cmd("config")
                                        .arg("set")
                                        .arg("port")
                                        .arg(format!("700{}", i))
                                        .query(&mut conection_client);
                                        if received_3.is_err(){
                                            panic!("GG")
                                        }

            }
        }
        */

        println!("Fin test of stess");
        println!("Fin test of stess");
        println!("Fin test of stess");
        println!("Fin test of stess");
        println!("Fin test of stess");
        println!("Fin test of stess");
        println!("Fin test of stess");
    }
}
