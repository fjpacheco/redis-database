use crate::tcp_protocol::close_thread;
use crate::{communication::log_messages::LogMessage, native_types::RError};
use std::{
    io::{BufRead, BufReader, Lines, Write},
    net::{Shutdown, SocketAddr, SocketAddrV4, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
};
use std::{sync::mpsc::Sender, thread::JoinHandle};

use crate::joinable::Joinable;
use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::{redis_type::encode_netcat_input, ErrorStruct, RArray, RedisType};
use crate::tcp_protocol::client_atributes::status::Status;

use super::{client_atributes::client_fields::ClientFields, notifier::Notifier, Response};

pub struct ClientHandler {
    stream: TcpStream,
    pub fields: Arc<Mutex<ClientFields>>,
    in_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    out_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    response_snd: mpsc::Sender<Option<String>>,
    addr: SocketAddrV4,
    notifier: Notifier,
}

impl ClientHandler {
    pub fn new(
        stream_received: TcpStream,
        notifier: Notifier,
    ) -> Result<ClientHandler, ErrorStruct> {
        let c_notifier = notifier.clone();
        let in_stream = stream_received
            .try_clone()
            .map_err(|_| ErrorStruct::from(redis_messages::clone_socket()))?;
        let out_stream = stream_received
            .try_clone()
            .map_err(|_| ErrorStruct::from(redis_messages::clone_socket()))?;
        let address = get_peer(&stream_received)?;
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
            read_socket(in_stream, c_shared_fields, c_notifier, response_snd_clone)
        });

        let out_thread = thread::spawn(move || write_socket(out_stream, response_recv));

        Ok(ClientHandler {
            stream: stream_received,
            fields: shared_fields,
            in_thread: Some(in_thread),
            out_thread: Some(out_thread),
            response_snd,
            addr,
            notifier,
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
    notifier: Notifier,
    response_snd: mpsc::Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    let buf_reader_stream = BufReader::new(
        stream
            .try_clone()
            .map_err(|_| ErrorStruct::from(redis_messages::clone_socket()))?,
    );

    let status_while = listen_while_client(
        buf_reader_stream.lines(),
        &c_shared_fields,
        &notifier,
        response_snd,
    )
    .map_err(|error| {
        if error.severity().eq(&Some(&ErrorSeverity::ShutdownServer)) {
            notifier.force_shutdown_server(error.print_it());
        }
        error
    });

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
    notifier.send_log(LogMessage::client_off(
        get_peer(&stream)
            .map(|x| x.to_string())
            .unwrap_or_else(|_| "Not found IP client".into()),
    ))?;
    status_while
}

fn listen_while_client(
    mut lines: Lines<BufReader<TcpStream>>,
    c_shared_fields: &Arc<Mutex<ClientFields>>,
    notifier: &Notifier,
    response_snd: mpsc::Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    while let Some(received) = lines.next() {
        match received {
            Ok(input) => {
                if input.starts_with('*') {
                    process_command_redis(
                        input,
                        &mut lines,
                        c_shared_fields,
                        notifier,
                        &response_snd,
                    )?;
                } else {
                    process_other(input, c_shared_fields, notifier, &response_snd)?;
                }
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => {
                    return Ok(());
                }
                _ => {
                    return Err(ErrorStruct::new(
                        "ERR".to_string(),
                        format!("Error received in next line.\nDetail: {:?}", err),
                    ))
                }
            },
        }
    }
    Ok(())
}

fn process_command_redis(
    mut input: String,
    mut lines_buffer_reader: &mut Lines<BufReader<TcpStream>>,
    client_status: &Arc<Mutex<ClientFields>>,
    notifier: &Notifier,
    response_sender: &Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    input.remove(0);
    process_command_general(
        input,
        &mut lines_buffer_reader,
        client_status,
        notifier,
        response_sender,
    )
}

fn process_other(
    input: String,
    client_status: &Arc<Mutex<ClientFields>>,
    notifier: &Notifier,
    response_sender: &Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    let mut input_encoded = encode_netcat_input(input)?;
    input_encoded.remove(0);
    let mut lines = BufReader::new(input_encoded.as_bytes()).lines();
    let first_lecture = lines
        .next()
        .ok_or_else(|| ErrorStruct::from(redis_messages::empty_buffer()))?
        .map_err(|_| ErrorStruct::from(redis_messages::normal_error()))?;
    process_command_general(
        first_lecture,
        &mut lines,
        client_status,
        &notifier,
        response_sender,
    )
}

fn process_command_general<G>(
    first_lecture: String,
    lines_buffer_reader: &mut Lines<G>,
    client_status: &Arc<Mutex<ClientFields>>,
    notifier: &Notifier,
    response_sender: &Sender<Option<String>>,
) -> Result<(), ErrorStruct>
where
    G: BufRead,
{
    let command_vec = RArray::decode(first_lecture, lines_buffer_reader)?;
    let result = client_status
        .lock()
        .map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "client_status",
                ErrorSeverity::CloseClient,
            ))
        })?
        .is_allowed_to(&command_vec[0]);

    match result {
        Ok(()) => delegate_command(command_vec, client_status, notifier, response_sender),
        Err(error) => send_response(RError::encode(error), response_sender),
    }
}

fn delegate_command(
    command_received: Vec<String>,
    client_fields: &Arc<Mutex<ClientFields>>,
    notifier: &Notifier,
    response_sender: &Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    let command_received_initial = command_received.clone();
    let (sender, receiver): (mpsc::Sender<Response>, mpsc::Receiver<Response>) = mpsc::channel();
    notifier.send_command_delegator(Some((
        command_received,
        sender,
        Arc::clone(&client_fields),
    )))?;
    for response in receiver.iter() {
        match response {
            Ok(good_string) => {
                send_response(good_string, response_sender)?;
            }
            Err(error) => {
                if let Some(severity) = error.severity() {
                    match severity {
                        ErrorSeverity::Comunicate => {
                            send_response(RError::encode(error), response_sender)?
                        }
                        ErrorSeverity::CloseClient => return Err(error),
                        ErrorSeverity::ShutdownServer => {
                            notifier.force_shutdown_server(error.print_it());
                            return Err(error);
                        }
                    }
                }
            }
        }
    }
    notifier.notify_successful_shipment(&client_fields, command_received_initial)?;
    Ok(())
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
    .ok_or_else(|| {
        ErrorStruct::from(redis_messages::init_failed(
            "new client",
            ErrorSeverity::CloseClient,
        ))
    })
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
        let _ = self.stream.shutdown(Shutdown::Both);
        let _ = self.response_snd.send(None);

        let state = close_thread(
            self.out_thread.take(),
            "write socket",
            self.notifier.clone(),
        )
        .and(close_thread(
            self.in_thread.take(),
            "read socket",
            self.notifier.clone(),
        ));

        println!("ME ELIMINE -- DROP//JOIN SUCCESS");
        state
    }
}
