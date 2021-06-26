use crate::tcp_protocol::client_atributes::client_status::ClientStatus;
use std::io::BufRead;
use std::io::Lines;
use std::net::Shutdown;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::{
    io::{BufReader, Write},
    net::TcpStream,
    sync::mpsc::Sender,
    thread,
};
//use std::collections::HashSet;

use crate::messages::redis_messages;
use crate::native_types::redis_type::encode_netcat_input;
use crate::native_types::{ErrorStruct, RArray, RError, RedisType};
use crate::tcp_protocol::client_atributes::status_answer::StatusAnswer;

pub struct ClientHandler {
    stream: TcpStream,
    _client_status: Arc<Mutex<ClientStatus>>,
}

impl ClientHandler {
    pub fn new(
        stream_received: TcpStream,
        command_delegator_sender: Sender<(Vec<String>, Sender<String>)>,
    ) -> ClientHandler {
        /*stream_received // FOR TIMEOUT OF REDIS.CONF
        .set_read_timeout(Some(Duration::new(5, 0)))
        .expect("set_read_timeout call failed");*/
        let mut stream = stream_received.try_clone().unwrap();
        let client_status = Arc::new(Mutex::new(ClientStatus::new()));
        let client_status_clone = Arc::clone(&client_status);

        let _client_thread = thread::spawn(move || {
            let buf_reader_stream = BufReader::new(stream.try_clone().unwrap());
            let mut linesbuffer_reader = buf_reader_stream.lines();
            let mut response;

            while let Some(received) = linesbuffer_reader.next() {
                match received {
                    Ok(mut input) => {
                        if input.starts_with('*') {
                            // example *3
                            input.remove(0); // i want -> 3
                            response = process(
                                input,
                                &mut linesbuffer_reader,
                                &command_delegator_sender,
                                Arc::clone(&client_status_clone),
                            );
                        } else {
                            let mut input_encoded = encode_netcat_input(input);
                            input_encoded.remove(0);
                            let mut lines = BufReader::new(input_encoded.as_bytes()).lines();
                            let first_lecture =
                                lines.next().unwrap().unwrap_or_else(|_| "-1".into());
                            response = process(
                                first_lecture,
                                &mut lines,
                                &command_delegator_sender,
                                Arc::clone(&client_status_clone),
                            );
                        }
                    }
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::WouldBlock => break, // FOR TIMEOUT OF REDIS.CONF
                        _ => {
                            response = RError::encode(ErrorStruct::new(
                                "ERR".to_string(),
                                format!("Error received in next line.\nDetail: {:?}", err),
                            ));
                        }
                    },
                }
                stream.write_all(response.as_bytes()).unwrap();
            }
            println!("<Server>: Loop finish. Client disconnected 💩 ");
        });

        ClientHandler {
            stream: stream_received,
            _client_status: client_status,
        }
    }

    pub fn send_response(&mut self, response: String) -> Result<(), ErrorStruct> {
        match self.stream.write_all(response.as_bytes()) {
            Ok(()) => Ok(()),
            Err(_) => Err(ErrorStruct::new(
                redis_messages::cannot_write_stream().get_prefix(),
                redis_messages::cannot_write_stream().get_message(),
            )),
        }
    }
}

fn process<G>(
    first_lecture: String,
    linesbuffer_reader: &mut Lines<G>,
    command_delegator_sender: &Sender<(Vec<String>, Sender<String>)>,
    client_status: Arc<Mutex<ClientStatus>>,
) -> String
where
    G: BufRead,
{
    match RArray::decode(first_lecture, linesbuffer_reader) {
        Ok(command_vec) => {
            println!(
                "<Server> Command: {:?}\n --- Encoded: {:?}\n",
                command_vec,
                RArray::encode(command_vec.clone())
            );

            match client_status.lock().unwrap().review_command(command_vec) {
                StatusAnswer::Continue(command_vec) => {
                    return delegate_command(command_vec, command_delegator_sender);
                }
                StatusAnswer::Break(some_error) => {
                    return RError::encode(some_error);
                }
                StatusAnswer::Done(result) => {
                    return encode_result(result);
                }
            }
        }
        Err(error) => {
            print!("Error decode in client handler");
            return RError::encode(error);
        }
    }

    fn encode_result(result: Result<String, ErrorStruct>) -> String {
        match result {
            Ok(encoded_resp) => encoded_resp,
            Err(err) => RError::encode(err),
        }
    }

    fn delegate_command(
        command_vec: Vec<String>,
        command_delegator_sender: &Sender<(Vec<String>, Sender<String>)>,
    ) -> String {
        let (sender, receiver): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
        command_delegator_sender
            .send((command_vec, sender))
            .unwrap();

        match receiver.recv() {
            Ok(encoded_resp) => encoded_resp,
            Err(err) => {
                let error = ErrorStruct::new(
                    "ERR".to_string(),
                    format!("failed to receive channel content. Detail {:?}", err),
                );
                RError::encode(error)
            }
        }
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        println!("Dropping ClienHandler 😜");
        self.stream
            .shutdown(Shutdown::Both)
            .expect("Error to close TcpStream");
    }
}
