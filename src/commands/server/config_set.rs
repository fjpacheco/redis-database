use crate::{commands::Runnable, native_types::ErrorStruct, tcp_protocol::server::ServerRedis};

pub struct ConfigSet;

impl Runnable<ServerRedis> for ConfigSet {
    fn run(
        &self,
        _buffer_vec: Vec<&str>,
        _server: &mut ServerRedis,
    ) -> Result<String, ErrorStruct> {
        Ok("+😜\r\n".into())
        // Acá SOLAMENTE TRABAJO CON EL COMANDO "CONFIG SET PORT" .. Ejemplo "config set port 9000"
        // Te cambiará todo al port 9000, y se muere el anterior port con el listener viejo!!
        // TODO ESTO se hace gracias al metodo de ListenerProcesosr::new_port()

        /*let new_port = match buffer_vec.get(1) {
            Some(item) => item.to_string(),
            None => {
                return Err(ErrorStruct::new(
                    "ERR_PORT".to_string(),
                    "Port not found in input".to_string(),
                ))
            }
        };

        server.new_port(new_port);
        */
    }
}
