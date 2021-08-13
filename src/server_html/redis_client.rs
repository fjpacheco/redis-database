use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Lines, Read, Write},
    net::{Shutdown, TcpStream},
    ops::Not,
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
    /// Creates the structure in charge of processing/parsing the website
    /// input received from the http server.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * There's an error when connecting to the server.
    /// * Streams clone can't be performed.
    /// * An operation processed by function process_web_input fails.
    pub fn new(
        available_commands: HashSet<String>,
        cmd_sender: Sender<Option<String>>, // Agus sender
        response_http_sender: Sender<Result<String, ErrorStruct>>, // Send OK or ERROR to Agus 77777
        http_receiver: Receiver<Option<String>>, // Agus sends me stuff and I receive it here xD
        address: String,
    ) -> Result<Self, ErrorStruct> {
        // address: self.ip.to_string() + ":" + &self.port
        let stream = TcpStream::connect(address).map_err(|_| {
            ErrorStruct::new(
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

        let http_sender_clone = response_http_sender.clone();

        let read_thread =
            thread::spawn(move || process_db_response(http_sender_clone, &mut stream_read_clone));

        let write_thread = thread::spawn(move || {
            process_web_input(
                &available_commands,
                &mut stream_write_clone,
                http_receiver,
                response_http_sender,
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
fn turn_into_vector(
    available_commands: &HashSet<String>,
    input: String,
) -> Result<Vec<String>, ErrorStruct> {
    let checked_input: String = convert_special_characters(input)?;
    let input_vector: Vec<String> = checked_input.split(' ').map(str::to_string).collect();
    if input_vector.len() == 0 || available_commands.contains(&input_vector[0]).not() {
        return Err(ErrorStruct::new(
            "COMMAND".to_string(),
            "User input did not match any available web command.".to_string(),
        ));
    }
    Ok(input_vector)
}

/// Returns the char related to the string hexadecimal.
fn hex_to_char(s: &str) -> Result<char, ErrorStruct> {
    match u8::from_str_radix(s, 16).map(|n| n as char) {
        Ok(char) => Ok(char),
        Err(_) => Err(ErrorStruct::new(
            "INPUT".to_string(),
            "User input had invalid characters.".to_string(),
        )),
    }
}

/// Checks for special characters on the string and replaces them.
/// Iterates through the input string looking for a '%', in case of
/// finding it, checks its next byte and obtains an ascii symbol,
/// then replaces it at the string.
pub fn convert_special_characters(input: String) -> Result<String, ErrorStruct> {
    let mut checked_input: Vec<String> = [].to_vec();
    let mut i = 0;
    let mut iter = input.chars();
    while let Some(mut char) = iter.next() {
        match char {
            '%' => {
                // checking for equal should be enough, as the loop ensures it
                if (i + 1) >= input.len() || (i + 2) >= input.len() {
                    return Err(ErrorStruct::new(
                        "INPUT_CHARACTERS".to_string(),
                        "Input contains non-interpretable characters.".to_string(),
                    ));
                } else {
                    let input_slice = input.get(i + 1..i + 3).unwrap();
                    char = hex_to_char(input_slice)?;
                    if char.to_string().is_ascii() {
                        i += 3;
                        iter.next();
                        iter.next();
                    } else {
                        return Err(ErrorStruct::new(
                            "INPUT_CHARACTERS".to_string(),
                            "Input contains non-interpretable characters.".to_string(),
                        ));
                    }
                }
            }
            _ => {
                i += 1;
            }
        }
        checked_input.push(char.to_string());
    }
    Ok(checked_input.into_iter().collect())
}

/// Iterates receiving string commands from the http server. If the command is
/// valid, writes it to the stream, if it's not, sends an error to the http server.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// * A closed channel won't let send a message to the http server.
/// * It is not possible to read a line from a stream.
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
/// * A closed channel won't let send a message to the http server.
/// * It is not possible to write a line to a stream.
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
    use std::{sync::mpsc, thread::{sleep, spawn, JoinHandle}, time::Duration};
    use crate::{ServerRedis, vec_strings};


    #[test]
    fn test_01() -> Result<(), ErrorStruct> {

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
        let server_thread: JoinHandle<Result<(), ErrorStruct>> = spawn(move || {
            ServerRedis::start(vec![])?;
            Ok(())
        });
        let millisecond = Duration::from_millis(10);
        let mut retries = 0;
        loop {
            let (response_http_mock, processed_response_mock) = mpsc::channel();
            let (cmd_sender_mock, http_receiver_mock) = mpsc::channel();
            match RedisClient::new(
                available_commands_set.clone(),
                cmd_sender_mock.clone(),
                response_http_mock,
                http_receiver_mock,
                "127.0.0.1:6379".to_string(),
            ) {
                Err(err) => {
                    if let Some(prefix) = err.prefix()  {
                        if prefix == "CONNECTION_FAIL" {
                            sleep(millisecond);
                            retries += 1;
                            if retries > 100000 {
                                return Err(ErrorStruct::new(
                                    "ERR_CLIENT".to_string(),
                                    "Tried to connect too many times".to_string()),
                                );
                            }  
                        } 
                    } else {
                        return Err(ErrorStruct::new(
                            "ERR_CLIENT".to_string(),
                            "Could not connect".to_string()),
                        );
                    }
                }
                Ok(redis_client) => {
                    cmd_sender_mock.send(Some("set key value1".to_string()));
                    dbg!(processed_response_mock.recv().unwrap());

                    cmd_sender_mock.send(Some("set key value2".to_string()));
                    dbg!(processed_response_mock.recv().unwrap());

                    cmd_sender_mock.send(Some("get key".to_string()));
                    dbg!(processed_response_mock.recv().unwrap());

                    cmd_sender_mock.send(Some("monitor".to_string()));
                    dbg!(processed_response_mock.recv().unwrap());

                    break;
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(
            convert_special_characters("example%3f".to_string()).unwrap(),
            "example?".to_string()
        );
        assert_eq!(
            convert_special_characters("example%3fexample".to_string()).unwrap(),
            "example?example".to_string()
        );
        assert_eq!(
            convert_special_characters("%3f".to_string()).unwrap(),
            "?".to_string()
        );
        assert_eq!(
            convert_special_characters("example".to_string()).unwrap(),
            "example".to_string()
        );
        assert_eq!(
            convert_special_characters("%253f".to_string()).unwrap(),
            "%3f".to_string()
        );
        assert_eq!(
            convert_special_characters("%252525252F".to_string()).unwrap(),
            "%2525252F".to_string()
        );
        assert_eq!(
            convert_special_characters("%2525%25".to_string()).unwrap(),
            "%25%".to_string()
        );

        assert_eq!(
            convert_special_characters("example%2".to_string())
                .unwrap_err()
                .print_it(),
            "INPUT_CHARACTERS Input contains non-interpretable characters.".to_string()
        );

        assert_eq!(
            convert_special_characters("example%".to_string())
                .unwrap_err()
                .print_it(),
            "INPUT_CHARACTERS Input contains non-interpretable characters.".to_string()
        );
    }
}
