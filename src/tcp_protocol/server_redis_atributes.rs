use std::fmt;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::ErrorStruct;
use crate::redis_config::RedisConfig;
use crate::tcp_protocol::client_list::ClientList;

#[derive(Clone)]
pub struct ServerRedisAtributes {
    config: Arc<Mutex<RedisConfig>>,
    status_listener: Arc<AtomicBool>,
    shared_clients: Arc<Mutex<ClientList>>,
}

impl ServerRedisAtributes {
    pub fn new(
        config: Arc<Mutex<RedisConfig>>,
        status_listener: Arc<AtomicBool>,
        shared_clients: Arc<Mutex<ClientList>>,
    ) -> Self {
        ServerRedisAtributes {
            config,
            status_listener,
            shared_clients,
        }
    }

    pub fn info(&mut self) -> Result<Vec<String>, ErrorStruct> {
        let mut info = Vec::new();

        self.config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "redis config",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .info(&mut info);

        self.shared_clients
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "redis config",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .info(&mut info);

        println!("{:?}", info);

        Ok(info)
    }

    pub fn get_client_list(&self) -> Arc<Mutex<ClientList>> {
        Arc::clone(&self.shared_clients)
    }

    pub fn store(&self, val: bool) {
        self.status_listener.store(val, Ordering::SeqCst);
    }
    pub fn change_verbose(&self, new: usize) -> Result<(), ErrorStruct> {
        self.config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .change_verbose(new);
        Ok(())
    }

    pub fn change_logfilename(&self, new_file_name: String) -> Result<(), ErrorStruct> {
        self.config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .change_log_file(new_file_name)?;
        Ok(())
    }

    pub fn set_timeout(&self, client: &TcpStream) -> Result<(), ErrorStruct> {
        let time = self
            .config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .timeout();
        if time.gt(&0) {
            client
                .set_read_timeout(Some(Duration::new(time, 0)))
                .map_err(|_| {
                    ErrorStruct::from(redis_messages::init_failed(
                        "Failed timeout",
                        ErrorSeverity::ShutdownServer,
                    ))
                })?;
        }
        Ok(())
    }

    pub fn get_addr(&self) -> Result<String, ErrorStruct> {
        Ok(self
            .config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .get_addr())
    }

    pub fn get_port(&self) -> Result<String, ErrorStruct> {
        Ok(self
            .config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .port())
    }

    pub fn get_verbose(&self) -> Result<String, ErrorStruct> {
        Ok(self
            .config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .verbose()
            .to_string())
    }

    pub fn get_timeout(&self) -> Result<String, ErrorStruct> {
        Ok(self
            .config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .timeout()
            .to_string())
    }

    pub fn get_logfile_name(&self) -> Result<String, ErrorStruct> {
        Ok(self
            .config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .log_filename())
    }

    pub fn get_dbfile_name(&self) -> Result<String, ErrorStruct> {
        Ok(self
            .config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .db_filename())
    }

    pub fn is_listener_off(&self) -> bool {
        self.status_listener
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl fmt::Display for ServerRedisAtributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server Redis Atributes")
    }
}
