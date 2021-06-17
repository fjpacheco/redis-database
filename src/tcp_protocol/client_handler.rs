use std::io::BufRead;
use std::io::Lines;
use std::sync::mpsc;
use std::{
    io::{BufReader, Write},
    net::TcpStream,
    sync::mpsc::Sender,
    thread,
};

use crate::native_types::redis_type::encode_netcat_input;
use crate::native_types::{ErrorStruct, RArray, RError, RedisType};

#[derive(Debug)]
pub struct ClientHandler;

impl ClientHandler {
    pub fn new(
        stream: TcpStream,
        // clients: Arc<Mutex<HashMap<SocketAddr, ClientHandler>>>,
        command_delegator_sender: Sender<(Vec<String>, Sender<String>)>,
    ) -> ClientHandler {
        //forma_original(stream, command_delegator_sender, &clients)
        forma_nueva(stream, command_delegator_sender)
    }
}

fn forma_nueva(
    stream: TcpStream,
    command_delegator_sender: Sender<(Vec<String>, Sender<String>)>,
) -> ClientHandler {
    let _client_thread = thread::spawn(move || {
        let mut _response = String::new();

        let buf_reader_stream = BufReader::new(stream.try_clone().unwrap());
        let mut stream_write = stream.try_clone().unwrap();

        let mut lines_buffer_reader = buf_reader_stream.lines();
        while let Some(received) = lines_buffer_reader.next() {
            if let Ok(mut input) = received {
                if input.starts_with('*') {
                    // example *3
                    input.remove(0); // i want -> 3
                    _response = process(input, &mut lines_buffer_reader, &command_delegator_sender);
                } else {
                    let mut input_encoded = encode_netcat_input(input);
                    input_encoded.remove(0);
                    let mut lines = BufReader::new(input_encoded.as_bytes()).lines();
                    let first_lecture = lines.next().unwrap().unwrap_or_else(|_| "-1".into());
                    _response = process(first_lecture, &mut lines, &command_delegator_sender);
                }
            } else {
                let error = ErrorStruct::new(
                    "ERR".to_string(),
                    "command received was not an array".to_string(),
                );
                _response = RError::encode(error);
            }
            stream_write.write_all(_response.as_bytes()).unwrap();
        }
        println!("<Server>: Loop finish. Client disconnected");
    });
    ClientHandler {}
}

fn process<G>(
    first_lecture: String,
    lines_buffer_reader: &mut Lines<G>,
    command_delegator_sender: &Sender<(Vec<String>, Sender<String>)>,
) -> String
where
    G: BufRead,
{
    let (sender, receiver): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
    match RArray::decode(first_lecture, lines_buffer_reader) {
        Ok(command_vec) => {
            println!(
                "<Server> Command: {:?}\n --- Encoded: {:?}\n",
                command_vec,
                RArray::encode(command_vec.clone())
            );
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
        Err(error) => {
            print!("Error decode in client handler");
            RError::encode(error)
        }
    }
}
