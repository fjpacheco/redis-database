use std::net::TcpListener;

use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    communication::log_messages::LogMessage,
    messages::redis_messages,
    messages::redis_messages::redis_logo,
    native_types::{error_severity::ErrorSeverity, ErrorStruct},
    redis_config::RedisConfig,
    tcp_protocol::client_handler::ClientHandler,
};

use super::notifier::Notifier;

/// Structure in charge of listening to new clients that connect to the server.
pub struct ListenerProcessor;

impl ListenerProcessor {
    /// Listen for connections from new clients connected to the server.
    /// If an error occurs, the server is forced to shut down in addition to writing the problem that occurred in the logs.
    pub fn incoming(
        listener: TcpListener,
        server_redis: ServerRedisAttributes,
        notifier: Notifier,
    ) {
        let result = start_incoming(listener, &notifier, server_redis);
        if let Err(err) = result {
            if err.severity().eq(&Some(&ErrorSeverity::ShutdownServer)) {
                notifier.force_shutdown_server("Forced shutdown".to_string());
            }
        }
    }

    /// Creates a new [TcpListener] which will be bound to the specified [RedisConfig].
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Cannot connect to requested address.
    /// * wrong address received in [RedisConfig].
    pub fn new_tcp_listener(config: &RedisConfig) -> Result<TcpListener, ErrorStruct> {
        let ip = config.ip();
        let port = config.port();
        let listener = Self::bind(&ip, &port)?;
        Ok(listener)
    }

    /// Creates a new [TcpListener] which will be bound to the specified ip and port.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Cannot connect to requested address.
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

/// The connections of new clients connected to the server will be listened to.
/// Each new connected client will have a new [ClientHandler] that will be stored
/// in the [ClientList] of [ServerRedisAttributes]. In addition, each customer
/// will be assigned the corresponding timeout.
/// It also informs the registries about the connection.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// * The channel to communicate with the [LogCenter] is closed.
/// * [ServerRedisAttributes] has poisoned fields.
fn start_incoming(
    listener: TcpListener,
    notifier: &Notifier,
    server_redis: ServerRedisAttributes,
) -> Result<(), ErrorStruct> {
    welcome_message(&listener, &notifier)?;
    for stream in listener.incoming() {
        if server_redis.status_listener() {
            break;
        }

        match stream {
            Ok(client) => {
                server_redis.set_timeout(&client)?;
                notifier.send_log(LogMessage::new_conection(&client))?;
                if let Ok(new_client) = ClientHandler::new(client, notifier.clone()) {
                    if let Ok(mut client_list) = server_redis.get_client_list().lock() {
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

///Print the welcome message with server details. Inform the logs of the server startup.
/// # Error
/// Returns an [ErrorStruct] if:
///
/// * The channel to communicate with the [LogCenter] is closed.
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
