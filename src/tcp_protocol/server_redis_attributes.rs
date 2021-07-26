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
pub struct ServerRedisAttributes {
    config: Arc<Mutex<RedisConfig>>,
    status_listener: Arc<AtomicBool>,
    shared_clients: Arc<Mutex<ClientList>>,
}

impl ServerRedisAttributes {
    pub fn new(
        config: Arc<Mutex<RedisConfig>>,
        status_listener: Arc<AtomicBool>,
        shared_clients: Arc<Mutex<ClientList>>,
    ) -> Self {
        ServerRedisAttributes {
            config,
            status_listener,
            shared_clients,
        }
    }

    /// Obtains in a [Vec]<[String]> detailed information about the status of the server with its clients, database and pubsub system.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * any structure to which the information is consulted is poisoned.
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

        Ok(info)
    }

    /// Returns a clone of [Arc]<[Mutex]<[ClientList](crate::tcp_protocol::client_list::ClientList)>> to be shared.
    pub fn get_client_list(&self) -> Arc<Mutex<ClientList>> {
        Arc::clone(&self.shared_clients)
    }

    /// Changes the state of the client with a [bool].
    ///
    /// * If [true]: stop listening to new clients with [TcpListener](std::net::TcpListener).
    /// * If [false]: keep listening to clients with [TcpListener](std::net::TcpListener).
    pub fn store(&self, val: bool) {
        self.status_listener.store(val, Ordering::SeqCst);
    }

    /// Change verbose level to display more or less debug information.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the name is poisoned.
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

    /// Change the name of the log file used to store debug information.
    //
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the name is poisoned.
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

    /// Change the name of the dump file used to store db information.
    //
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the name is poisoned.
    pub fn change_dump_filename(&self, new_file_name: String) -> Result<(), ErrorStruct> {
        self.config
            .lock()
            .map_err(|_| {
                ErrorStruct::from(redis_messages::poisoned_lock(
                    "Server Redis Atributes",
                    ErrorSeverity::ShutdownServer,
                ))
            })?
            .change_dump_file(new_file_name)?;
        Ok(())
    }

    /// Changes the time it takes to disconnect a client that is not interacting with the server.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the timeout is poisoned.    
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

    /// Gets a [String] with the address to connect as a client to the server.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the address is poisoned.
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

    /// Gets a [String] with the port to connect as a client to the server.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the port is poisoned.
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

    /// Gets a [String] with the verbosity level to display debug information.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the verbose level is poisoned.
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

    /// Gets a [String] with timeout allowed for clients.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the timeout is poisoned.
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

    /// Gets a [String] with the name of the file in charge of saving the debug information.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the name is poisoned.
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

    /// Gets a [String] with the name of the file in charge of the persistence of the [Database](crate::database::Database).
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * the structure that stores the name is poisoned.
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

    /// Returns the current state of the listener processor with [bool].
    ///
    /// * If [true]: stop listening to new clients with [TcpListener](std::net::TcpListener).
    /// * If [false]: keep listening to clients with [TcpListener](std::net::TcpListener).
    pub fn status_listener(&self) -> bool {
        self.status_listener
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}
