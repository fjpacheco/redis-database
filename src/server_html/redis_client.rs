use std::ops::Not;
use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Lines, Read, Write},
    net::{Shutdown, TcpStream},
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    joinable::Joinable,
    messages::redis_messages,
    native_types::{ErrorStruct, RArray, RBulkString, RError, RInteger, RSimpleString, RedisType},
};

/// Manages the website input.
/// The input can be delegated to the database to be processed or could be
/// incorrect, and throw an error in case the command is not available at
/// the website or does not exist.
pub struct RedisClient {
    cmd_sender: Sender<Option<String>>,
    stream: TcpStream,
    read_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    write_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
}

impl RedisClient {
    /// Creates the structure in charge of processing the website input.
    ///
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * TODO: COMPLETE THIS
    pub fn new(
        available_commands: HashSet<String>,
        cmd_sender: Sender<Option<String>>,
        http_sender: Sender<Result<String, ErrorStruct>>,
        cmd_receiver: Receiver<Option<String>>,
        address: String,
    ) -> Result<Self, ErrorStruct> {
        let stream = TcpStream::connect(address).map_err(|_| {
            ErrorStruct::new(
                // address = self.ip.to_string() + ":" + &self.port
                "CONNECTION_FAIL".to_string(),
                "Redis Client couldn't connect to the server.".to_string(),
            )
        })?;

        let mut stream_read_clone = stream
            .try_clone()
            .map_err(|_| ErrorStruct::new("TODO".to_string(), "TODO".to_string()))?;

        let mut stream_write_clone = stream
            .try_clone()
            .map_err(|_| ErrorStruct::new("TODO".to_string(), "TODO".to_string()))?;

        let http_sender_clone = http_sender.clone();

        let read_thread =
            thread::spawn(move || process_db_response(http_sender_clone, &mut stream_read_clone));

        let write_thread = thread::spawn(move || {
            process_web_input(
                &available_commands,
                &mut stream_write_clone,
                cmd_receiver,
                http_sender,
            )
        });

        Ok(RedisClient {
            cmd_sender,
            stream,
            read_thread: Some(read_thread),
            write_thread: Some(write_thread),
        })
    }
}

/// Checks if the input received as string relates to an available command.
/// If it does, returns a vector with the string split by whitespace, if not
/// returns error.
pub fn turn_into_vector(
    available_commands: &HashSet<String>,
    input: String,
) -> Result<Vec<String>, ErrorStruct> {
    // TODO: here could be called a function to check for special characters and replace them
    let input_vector: Vec<String> = input.split(' ').map(str::to_string).collect();
    if input_vector.len() == 0 || available_commands.contains(&input_vector[0]).not() {
        return Err(ErrorStruct::new(
            "COMMAND".to_string(),
            "User input did not match any available web command.".to_string(),
        ));
    }
    Ok(input_vector)
}

/// Iterates receiving string commands from the http server. If the command is
/// valid, writes it to the stream, if it's not, sends an error to the http server.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// *
fn process_db_response(
    http_sender: Sender<Result<String, ErrorStruct>>,
    stream: &mut impl Read,
) -> Result<(), ErrorStruct> {
    let stream_reader = BufReader::new(stream);
    let mut lines = stream_reader.lines();
    while let Some(received) = lines.next() {
        match received {
            Ok(db_response) => {
                let string_response = get_string_response(db_response, &mut lines);
                http_sender.send(string_response).map_err(|_| {
                    ErrorStruct::new(
                        "CLOSED_CHANNEL".to_string(),
                        "Message couldn't be delivered to http server.".to_string(),
                    )
                })?;
            }
            Err(_) => {
                return Err(ErrorStruct::new(
                    "STREAM_READ".to_string(),
                    "Error while reading line from stream.".to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Decodes the response receveid according to its first character using RedisType decode method
/// and returns a string representation of it. In case of error, returns an ErrorStruct.
fn get_string_response(
    mut db_response: String,
    mut lines: &mut Lines<BufReader<&mut impl Read>>,
) -> Result<String, ErrorStruct> {
    let string_response;
    match db_response.remove(0) {
        '*' => {
            let array_response = RArray::decode(db_response, &mut lines);
            string_response = array_response.map(|a| a.join("\n"));
        }
        '+' => {
            string_response = RSimpleString::decode(db_response, &mut lines);
        }
        '-' => {
            let error_response = RError::decode(db_response, &mut lines);
            string_response = error_response.map(|a| a.print_it());
        }
        ':' => {
            let integer_response = RInteger::decode(db_response, &mut lines);
            string_response = integer_response.map(|a| a.to_string());
        }
        '$' => {
            string_response = RBulkString::decode(db_response, &mut lines);
        }
        _ => {
            return Err(ErrorStruct::new(
                "STREAM_READ".to_string(),
                "Error while reading line from stream.".to_string(),
            ));
        }
    }
    string_response
}

/// Iterates receiving string commands from the http server. If the command is
/// valid, writes it to the stream, if it's not, sends an error to the http server.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// *
fn process_web_input(
    available_commands: &HashSet<String>,
    stream: &mut impl Write,
    http_receiver: Receiver<Option<String>>,
    http_sender: Sender<Result<String, ErrorStruct>>,
) -> Result<(), ErrorStruct> {
    for wrapped_cmd in http_receiver.iter() {
        if let Some(cmd) = wrapped_cmd {
            match turn_into_vector(available_commands, cmd) {
                Ok(cmd) => {
                    let encoded_cmd_vector = RArray::encode(cmd);
                    stream
                        .write_all(encoded_cmd_vector.as_bytes())
                        .map_err(|_| {
                            ErrorStruct::new(
                                "STREAM_WRITE".to_string(),
                                "Stream write couldn't be performed successfully.".to_string(),
                            )
                        })?;
                }
                Err(err) => {
                    // in case of error http server must be warned!
                    http_sender.send(Err(err)).map_err(|_| {
                        ErrorStruct::new(
                            "CLOSED_CHANNEL".to_string(),
                            "Message couldn't be delivered to http server.".to_string(),
                        )
                    })?;
                }
            }
        } else {
            break;
        }
    }
    Ok(())
}

impl Joinable<()> for RedisClient {
    fn join(&mut self) -> Result<(), ErrorStruct> {
        let _ = self.stream.shutdown(Shutdown::Both);
        let _ = self.cmd_sender.send(None);
        let _ = close_thread(self.read_thread.take(), "redis client read thread");
        let _ = close_thread(self.write_thread.take(), "redis client write thread");
        Ok(())
    }
}

pub fn close_thread(
    thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    name: &str,
) -> Result<(), ErrorStruct> {
    if let Some(handle) = thread {
        handle
            .join()
            .map_err(|_| ErrorStruct::from(redis_messages::thread_panic(name)))?
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test_redis_client {

    use super::*;
    use crate::vec_strings;
    use std::sync::mpsc;

    #[test]
    fn test_01() {
        let available_commands_list: Vec<String> = vec_strings![
            "decrby",
            "del",
            "exists",
            "get",
            "getset",
            "incrby",
            "keys",
            "lindex",
            "llen",
            "lpop",
            "lpush",
            "lrange",
            "lrem",
            "lset",
            "mget",
            "mset",
            "rename",
            "rpop",
            "rpush",
            "sadd",
            "scard",
            "set",
            "sismember",
            "smembers",
            "sort",
            "srem",
            "ttl",
            "type"
        ];
        let available_commands_set: HashSet<String> = available_commands_list
            .iter()
            .map(|member| member.to_string())
            .collect();
        let (http_sender_mock, _) = mpsc::channel();
        let (cmd_sender_mock, cmd_receiver_mock) = mpsc::channel();
        let _ = RedisClient::new(
            available_commands_set,
            cmd_sender_mock,
            http_sender_mock,
            cmd_receiver_mock,
            "127.0.0.1:6379".to_string(),
        );
    }
}
