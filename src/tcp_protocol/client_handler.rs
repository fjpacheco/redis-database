use std::{
    io::{BufRead, BufReader, Lines, Write},
    net::{Shutdown, SocketAddr, SocketAddrV4, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::native_types::{
    redis_type::encode_netcat_input, ErrorStruct, RArray, RError, RedisType,
};

use super::{
    client_atributes::{client_fields::ClientFields, status_answer::StatusAnswer},
    notifiers::Notifiers,
};

pub struct ClientHandler {
    stream: TcpStream,
    pub fields: Arc<Mutex<ClientFields>>,
}

impl ClientHandler {
    pub fn new(stream_received: TcpStream, notifiers: Notifiers) -> ClientHandler {
        let mut stream = stream_received.try_clone().unwrap();
        let address = get_peer(&mut stream).unwrap();
        let fields = ClientFields::new(address);
        let shared_fields = Arc::new(Mutex::new(fields));
        let c_shared_fields = Arc::clone(&shared_fields);

        let _client_thread = thread::spawn(move || {
            process_client(stream, c_shared_fields, notifiers);
        });

        ClientHandler {
            stream: stream_received,
            fields: shared_fields,
        }
    }

    pub fn is_monitor_notifiable(&self) -> bool {
        self.fields.lock().unwrap().is_monitor_notifiable()
    }

    pub fn get_peer(&mut self) -> Option<SocketAddrV4> {
        get_peer(&mut self.stream)
    }

    pub fn write_stream(&mut self, response: String) {
        write_stream(&mut self.stream, response);
    }
}

fn process_client(
    mut stream: TcpStream,
    c_shared_fields: Arc<Mutex<ClientFields>>,
    notifiers: Notifiers,
) {
    let buf_reader_stream = BufReader::new(stream.try_clone().unwrap());
    let mut lines_buffer_reader = buf_reader_stream.lines();
    let mut response;
    while let Some(received) = lines_buffer_reader.next() {
        match received {
            Ok(input) => {
                if input.starts_with('*') {
                    response = process_command_redis(
                        input,
                        &mut lines_buffer_reader,
                        Arc::clone(&c_shared_fields),
                        &notifiers,
                    );
                } else {
                    response =
                        process_command_string(input, Arc::clone(&c_shared_fields), &notifiers);
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
        write_stream(&mut stream, response);
    }
    notifiers.off_client(&stream);
}

fn write_stream(stream: &mut TcpStream, response: String) {
    stream.write_all(response.as_bytes()).unwrap();
}

fn process_command_redis(
    mut input: String,
    mut lines_buffer_reader: &mut Lines<BufReader<TcpStream>>,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> String {
    input.remove(0);
    process_command_general(input, &mut lines_buffer_reader, client_status, notifiers)
}

fn process_command_string(
    input: String,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> String {
    let mut input_encoded = encode_netcat_input(input);
    input_encoded.remove(0);
    let mut lines = BufReader::new(input_encoded.as_bytes()).lines();
    let first_lecture = lines.next().unwrap().unwrap_or_else(|_| "-1".into());
    process_command_general(first_lecture, &mut lines, client_status, &notifiers)
}

fn process_command_general<G>(
    first_lecture: String,
    lines_buffer_reader: &mut Lines<G>,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> String
where
    G: BufRead,
{
    match RArray::decode(first_lecture, lines_buffer_reader) {
        Ok(command_vec) => {
            let answer = client_status.lock().unwrap().review_command(command_vec);

            match answer {
                StatusAnswer::Continue(command_vec) => {
                    delegate_command(command_vec, client_status, notifiers)
                }
                StatusAnswer::Break(some_error) => RError::encode(some_error),
                StatusAnswer::Done(result) => match result {
                    Ok(encoded_resp) => encoded_resp,
                    Err(err) => RError::encode(err),
                },
            }
        }
        Err(error) => RError::encode(error),
    }
}

fn delegate_command(
    command_received: Vec<String>,
    client_fields: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> String {
    let command_received_initial = command_received.clone();
    let (sender, receiver): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    let _a = notifiers.send_command_delegator(command_received, sender);

    match receiver.recv() {
        Ok(response) => {
            notifiers.notify_successful_shipment(client_fields, command_received_initial);
            response
        }
        Err(err) => RError::encode(ErrorStruct::new(
            "ERR".to_string(),
            format!("failed to receive channel content. Detail {:?}", err),
        )),
    }
}

pub fn get_peer(stream: &mut TcpStream) -> Option<SocketAddrV4> {
    match stream.peer_addr().unwrap() {
        SocketAddr::V4(addr) => Some(addr),
        SocketAddr::V6(_) => None,
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        self.stream
            .shutdown(Shutdown::Both)
            .expect("Error to close TcpStream");
    }
}
