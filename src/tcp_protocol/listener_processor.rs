use std::net::TcpListener;

use crate::{
    communication::log_messages::LogMessage,
    messages::redis_messages,
    messages::redis_messages::redis_logo,
    native_types::{error_severity::ErrorSeverity, ErrorStruct},
    redis_config::RedisConfig,
    tcp_protocol::client_handler::ClientHandler,
};

use super::{notifier::Notifier, server::ServerRedisAtributes};

pub struct ListenerProcessor;

impl ListenerProcessor {
    pub fn incoming(listener: TcpListener, server_redis: ServerRedisAtributes, notifier: Notifier) {
        let result = start_incoming(listener, &notifier, server_redis);
        if let Err(err) = result {
            if err.severity().eq(&Some(&ErrorSeverity::ShutdownServer)) {
                notifier.force_shutdown_server("Forced shutdown".to_string());
            }
        }
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

fn start_incoming(
    listener: TcpListener,
    notifier: &Notifier,
    server_redis: ServerRedisAtributes,
) -> Result<(), ErrorStruct> {
    welcome_message(&listener, &notifier)?;
    for stream in listener.incoming() {
        if server_redis.is_listener_off() {
            break;
        }

        match stream {
            Ok(client) => {
                server_redis.set_timeout(&client)?;
                notifier.send_log(LogMessage::new_conection(&client))?;
                if let Ok(new_client) = ClientHandler::new(client, notifier.clone()) {
                    if let Ok(mut client_list) = server_redis.shared_clients.lock() {
                        client_list.insert(new_client);
                    } else {
                        let _ = notifier.send_log(LogMessage::from_errorstruct(
                            // I'm not interested ... I retired with the forced Shutdown!
                            ErrorStruct::from(redis_messages::poisoned_lock(
                                "Client List",
                                ErrorSeverity::ShutdownServer,
                            )),
                        ))?;
                    }
                }
            }
            Err(e) => {
                notifier.send_log(LogMessage::error_to_connect_client(&e))?;
            }
        }
    }
    notifier.send_log(LogMessage::off_server(&listener))?;
    Ok(())
}

fn welcome_message(listener: &TcpListener, notifier: &Notifier) -> Result<(), ErrorStruct> {
    let port = listener
        .local_addr()
        .map_err(|_| {
            ErrorStruct::from(redis_messages::init_failed(
                "Fail in local address of listener",
                ErrorSeverity::ShutdownServer,
            ))
        })?
        .port()
        .to_string();
    print!("{}", redis_logo(&port));
    notifier.send_log(LogMessage::start_up(listener))?;
    Ok(())
}
