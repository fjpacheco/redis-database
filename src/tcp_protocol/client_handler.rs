use crate::tcp_protocol::close_thread;
use std::thread::JoinHandle;
use std::{
    io::{BufRead, BufReader, Lines, Write},
    net::{Shutdown, SocketAddr, SocketAddrV4, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::joinable::Joinable;
use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::{redis_type::encode_netcat_input, ErrorStruct, RArray, RedisType};
use crate::tcp_protocol::client_atributes::status::Status;

use super::{client_atributes::client_fields::ClientFields, notifiers::Notifiers};

pub struct ClientHandler {
    stream: TcpStream,
    pub fields: Arc<Mutex<ClientFields>>,
    in_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    out_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    response_snd: mpsc::Sender<Option<String>>,
    addr: SocketAddrV4,
}

impl ClientHandler {
    pub fn new(
        mut stream_received: TcpStream,
        notifiers: Notifiers,
    ) -> Result<ClientHandler, ErrorStruct> {
        let in_stream = stream_received
            .try_clone()
            .map_err(|_| ErrorStruct::from(redis_messages::clone_socket()))?;
        let out_stream = stream_received
            .try_clone()
            .map_err(|_| ErrorStruct::from(redis_messages::clone_socket()))?;
        let address = get_peer(&mut stream_received)?;
        let addr = address; /* Culpa de Martina */
        let fields = ClientFields::new(address);
        let shared_fields = Arc::new(Mutex::new(fields));
        let c_shared_fields = Arc::clone(&shared_fields);

        let (response_snd, response_recv): (
            mpsc::Sender<Option<String>>,
            mpsc::Receiver<Option<String>>,
        ) = mpsc::channel();
        let response_snd_clone = response_snd.clone();
        let in_thread = thread::spawn(move || {
            read_socket(in_stream, c_shared_fields, notifiers, response_snd_clone)
        });

        let out_thread = thread::spawn(move || write_socket(out_stream, response_recv));

        Ok(ClientHandler {
            stream: stream_received,
            fields: shared_fields,
            in_thread: Some(in_thread),
            out_thread: Some(out_thread),
            response_snd,
            addr,
        })
    }

    pub fn is_subscripted_to(&self, channel: &str) -> bool {
        if let Ok(fields_guard) = self.fields.lock() {
            return fields_guard.is_subscripted_to(channel);
        }

        false
    }

    pub fn is_monitor_notificable(&self) -> bool {
        if let Ok(fields_guard) = self.fields.lock() {
            return fields_guard.is_monitor_notificable();
        }

        false
    }

    pub fn is_dead(&self) -> bool {
        if let Ok(fields_guard) = self.fields.lock() {
            return fields_guard.is_dead();
        }

        true
    }

    pub fn get_peer(&self) -> Result<SocketAddrV4, ErrorStruct> {
        get_peer(&self.stream)
    }

    pub fn get_addr(&self) -> String {
        self.addr.clone().to_string()
    }

    pub fn write_stream(&self, response: String) -> Result<(), ErrorStruct> {
        send_response(response, &self.response_snd)
    }

    pub fn get_detail(&self) -> String {
        match self.fields.lock() {
            Ok(fields_guard) => fields_guard.get_detail(),
            Err(_) => String::from("(nil)"),
        }
    }
}

fn write_socket(
    mut stream: TcpStream,
    response_recv: mpsc::Receiver<Option<String>>,
) -> Result<(), ErrorStruct> {
    for packed_response in response_recv.iter() {
        if let Some(response) = packed_response {
            stream
                .write_all(response.as_bytes())
                .map_err(|_| ErrorStruct::from(redis_messages::closed_socket()))?;
        } else {
            return Ok(());
        }
    }

    Ok(())
}

fn read_socket(
    stream: TcpStream,
    c_shared_fields: Arc<Mutex<ClientFields>>,
    notifiers: Notifiers,
    response_snd: mpsc::Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    let buf_reader_stream = BufReader::new(
        stream
            .try_clone()
            .map_err(|_| ErrorStruct::from(redis_messages::clone_socket()))?,
    );
    let mut lines = buf_reader_stream.lines();
    let mut response;
    while let Some(received) = lines.next() {
        match received {
            Ok(input) => {
                let client_status = Arc::clone(&c_shared_fields);
                if input.starts_with('*') {
                    response = process_command_redis(input, &mut lines, client_status, &notifiers);
                } else {
                    response = process_command_string(input, client_status, &notifiers);
                }
            }
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::WouldBlock => break, // FOR TIMEOUT OF REDIS.CONF
                    _ => {
                        response = Err(ErrorStruct::new(
                            "ERR".to_string(),
                            format!("Error received in next line.\nDetail: {:?}", err),
                        ))
                    }
                }
            }
        }

        //TODO: ANALISIS DE UNWRAPS. EL PRIMER UNWRAP ES DEMASIADO CRITICO => PREFIX POCO UTIL, DEBATIR IDEA DE
        let response_str;
        match response {
            Ok(item) => {
                response_str = item;
            }
            Err(item) => {
                response_str = item.print_it();
            }
        }

        send_response(response_str, &response_snd)?;
    }
    notifiers.off_client(get_peer(&stream)?.to_string());
    println!("😢 UN CLIENTE SE FUE 😢 => LE PONGO STATUS DEAD :) ");
    c_shared_fields
        .lock()
        .map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "client_status",
                ErrorSeverity::CloseClient,
            ))
        })?
        .replace_status(Status::Dead);
    Ok(())
}

fn process_command_redis(
    mut input: String,
    mut lines_buffer_reader: &mut Lines<BufReader<TcpStream>>,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> Result<String, ErrorStruct> {
    input.remove(0);
    process_command_general(input, &mut lines_buffer_reader, client_status, notifiers)
}

fn process_command_string(
    input: String,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> Result<String, ErrorStruct> {
    let mut input_encoded = encode_netcat_input(input);
    input_encoded.remove(0);
    let mut lines = BufReader::new(input_encoded.as_bytes()).lines();
    let first_lecture = lines
        .next()
        .ok_or(ErrorStruct::from(redis_messages::empty_buffer()))?
        .map_err(|_| ErrorStruct::from(redis_messages::normal_error()))?;

    process_command_general(first_lecture, &mut lines, client_status, &notifiers)
}

fn process_command_general<G>(
    first_lecture: String,
    lines_buffer_reader: &mut Lines<G>,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> Result<String, ErrorStruct>
where
    G: BufRead,
{
    let command_vec = RArray::decode(first_lecture, lines_buffer_reader)?;
    client_status
        .lock()
        .map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "client_status",
                ErrorSeverity::CloseClient,
            ))
        })?
        .is_allowed_to(&command_vec[0])?;
    delegate_command(command_vec, client_status, notifiers)
}

fn delegate_command(
    command_received: Vec<String>,
    client_fields: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> Result<String, ErrorStruct> {
    let command_received_initial = command_received.clone();
    let (sender, receiver): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    notifiers.send_command_delegator((command_received, sender, Arc::clone(&client_fields)))?;

    let response = receiver
        .recv()
        .map_err(|_| ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::Comunicate)))?;

    notifiers.notify_successful_shipment(client_fields, command_received_initial)?;
    Ok(response)
}

pub fn get_peer(stream: &TcpStream) -> Result<SocketAddrV4, ErrorStruct> {
    match stream.peer_addr().map_err(|_| {
        ErrorStruct::from(redis_messages::init_failed(
            "new client",
            ErrorSeverity::CloseClient,
        ))
    })? {
        SocketAddr::V4(addr) => Some(addr),
        SocketAddr::V6(_) => None,
    }
    .ok_or(ErrorStruct::from(redis_messages::init_failed(
        "new client",
        ErrorSeverity::CloseClient,
    )))
}

fn send_response(
    response: String,
    sender: &mpsc::Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    sender
        .send(Some(response))
        .map_err(|_| ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::CloseClient)))
}

impl Joinable<()> for ClientHandler {
    fn join(&mut self) -> Result<(), ErrorStruct> {
        match self.stream.shutdown(Shutdown::Both) {
            Ok(()) => { /* Socket has been closed right now */ }
            Err(_) => { /* Socket is already closed */ }
        }

        match self.response_snd.send(None) {
            Ok(()) => { /* Channel has been closed right now */ }
            Err(_) => { /* Channel is already closed */ }
        }

        let state = close_thread(self.out_thread.take(), "write socket")
            .and(close_thread(self.in_thread.take(), "read socket"));

        println!("ME ELIMINE -- DROP//JOIN SUCCESS");
        state
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        let _ = self.join();
    }
}
