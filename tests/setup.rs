use std::{
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use redis_rust::{
    messages::redis_messages, native_types::ErrorStruct, redis_config::RedisConfig,
    tcp_protocol::server::ServerRedis,
};

/// Provee un servidor Redis (desarrollado por Rust-eze Team ©) para los tests de integración.
///
/// Además incluye el cliente del crate de Redis que se necesita para ejecutar los comandos del crate de Redis.
pub struct ServerTest {
    client: redis::Client,
    server_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
}

impl ServerTest {
    /// Levanta un servidor Redis (desarrollado por Rust-eze Team ©) para los tests de integración en un thread aparte.
    /// Conecta un cliente del crate de Redis que se necesita para ejecutar los comandos del crate de Redis.
    /// Siempre se inicia el server con la database vacía, limpia.
    pub fn start() -> Result<Self, ErrorStruct> {
        let server_thread: JoinHandle<Result<(), ErrorStruct>> = spawn(move || {
            ServerRedis::start(vec![])?;
            Ok(())
        });

        let client_redis = redis::Client::open(
            "redis://".to_owned()
                + &RedisConfig::default().ip()
                + ":"
                + &RedisConfig::default().port()
                + "/",
        )
        .map_err(|_| {
            ErrorStruct::new(
                "ERR_CLIENT".to_string(),
                "Failed conection of client.".to_string(),
            )
        })?;

        let mut connection;

        let millisecond = Duration::from_millis(10);
        let mut retries = 0;
        loop {
            match client_redis.get_connection() {
                Err(err) => {
                    if err.is_connection_refusal() {
                        sleep(millisecond);
                        retries += 1;
                        if retries > 100000 {
                            return Err(ErrorStruct::new(
                                "ERR_CLIENT".to_string(),
                                format!("Tried to connect too many times, last error: {}", err),
                            ));
                        }
                    } else {
                        return Err(ErrorStruct::new(
                            "ERR_CLIENT".to_string(),
                            format!("Could not connect: {}", err),
                        ));
                    }
                }
                Ok(x) => {
                    connection = x;
                    break;
                }
            }
        }

        redis::cmd("flushdb").execute(&mut connection);
        Ok(Self {
            client: client_redis,
            server_thread: Some(server_thread),
        })
    }

    /// Retorna la conexión del cliente con el crate de Redis para ejecutar los comandos del crate de Redis.
    pub fn get_connection_client(&self) -> Result<redis::Connection, ErrorStruct> {
        self.client.get_connection().map_err(|_| {
            ErrorStruct::new(
                "ERR_CLIENT".to_string(),
                "Failed conection of client.".to_string(),
            )
        })
    }

    /// Apaga el servidor con el comando "shutdown", previamente realiza una limpieza de la database.
    /// Libera la memoria del thread usado para el servidor de los tests de integración.
    pub fn shutdown(&mut self) -> Result<(), ErrorStruct> {
        redis::cmd("flushdb").execute(&mut self.client);
        redis::cmd("shutdown")
            .query(&mut self.client)
            .map_err(|_| {
                ErrorStruct::from(redis_messages::thread_panic("server for test integration"))
            })?;
        if let Some(handle) = self.server_thread.take() {
            handle.join().map(|_| ()).map_err(|_| {
                ErrorStruct::from(redis_messages::thread_panic("server for test integration"))
            })?;
        }
        Ok(())
    }
}
