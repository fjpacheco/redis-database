use std::net::TcpListener;

use crate::{
    communication::log_messages::LogMessage, messages::redis_messages::redis_logo,
    native_types::ErrorStruct, redis_config::RedisConfig,
    tcp_protocol::client_handler::ClientHandler,
};

use super::{notifiers::Notifiers, server::ServerRedisAtributes};

pub struct ListenerProcessor;

impl ListenerProcessor {
    pub fn incoming(
        listener: TcpListener,
        server_redis: ServerRedisAtributes,
        notifiers: Notifiers,
    ) {
        print!("{}", redis_logo(&server_redis.get_port()));
        let _ = notifiers.send_log(LogMessage::start_up(&listener));

        for stream in listener.incoming() {
            if server_redis.is_listener_off() {
                break;
            }

            match stream {
                Ok(client) => {
                    server_redis.set_timeout(&client);
                    let _ = notifiers.send_log(LogMessage::new_conection(&client));
                    let new_client = ClientHandler::new(client, notifiers.clone());
                    server_redis
                        .shared_clients
                        .lock()
                        .unwrap()
                        .insert(new_client);
                }
                Err(e) => {
                    let _ = notifiers.send_log(LogMessage::error_to_connect_client(&e));
                }
            }
        }
        let _ = notifiers.send_log(LogMessage::off_server(&listener));
    }

    pub fn new_tcp_listener(config: &RedisConfig) -> Result<TcpListener, ErrorStruct> {
        let ip = config.ip();
        let port = config.port();
        let listener = Self::bind(&ip, &port)?;
        Ok(listener)
    }

    fn bind(ip: &str, port: &str) -> Result<TcpListener, ErrorStruct> {
        match TcpListener::bind(ip.to_owned() + ":" + port) {
            Ok(listener) => Ok(listener),
            Err(error) => Err(ErrorStruct::new(
                "ERR_BIND".into(),
                format!("Bind failure. Detail: {}", error),
            )),
        }
    }
}
