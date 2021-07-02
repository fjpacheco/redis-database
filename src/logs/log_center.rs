use std::fs::OpenOptions;
use std::io::LineWriter;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::SystemTime;

use crate::communication::log_messages::LogMessage;
use crate::file_manager::FileManager;
use crate::native_types::error::ErrorStruct;
use crate::redis_config::RedisConfig;

pub struct FileHandler {
    last_message: Option<String>,
}

impl Default for FileHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl FileHandler {
    pub fn new() -> FileHandler {
        FileHandler { last_message: None }
    }

    fn _write_line(&mut self, line: String) {
        self.last_message = Some(line);
    }

    #[allow(dead_code)]
    fn get(&mut self) -> Option<String> {
        self.last_message.take()
    }
}

pub struct LogCenter {
    _handler: Option<JoinHandle<()>>,
}

impl LogCenter {
    pub fn new(
        receiver: Receiver<LogMessage>,
        redis_config: Arc<Mutex<RedisConfig>>,
        writer: FileManager,
    ) -> Result<LogCenter, ErrorStruct> {
        let builder = thread::Builder::new().name("Log Center".into());
        let log_handler = LogCenter::spawn_handler(builder, receiver, redis_config, writer)?;

        Ok(LogCenter {
            _handler: Some(log_handler),
        })
    }

    fn spawn_handler(
        builder: thread::Builder,
        receiver: Receiver<LogMessage>,
        redis_config: Arc<Mutex<RedisConfig>>,
        writer: FileManager,
    ) -> Result<JoinHandle<()>, ErrorStruct> {
        match builder.spawn(move || LogCenter::start(receiver, redis_config, writer)) {
            Ok(handler) => Ok(handler),
            Err(_) => Err(ErrorStruct::new(
                "INITFAILED".to_string(),
                "Failed to create Log Center".to_string(),
            )),
        }
    }

    fn start(
        receiver: Receiver<LogMessage>,
        redis_config: Arc<Mutex<RedisConfig>>,
        writer: FileManager,
    ) {
        // Open? Or create? D:
        let file = OpenOptions::new()
            .write(true)
            .open(&redis_config.lock().unwrap().log_filename())
            .unwrap();
        let mut file = LineWriter::new(file);
        // Considerar caso en el que cambia el archivo redis config
        for mut log_message in receiver.iter() {
            if let Some(message) =
                log_message.is_verbosely_printable(redis_config.lock().unwrap().verbose())
            {
                LogCenter::print_log_message(message);
            }

            writer
                .write_to_file(&mut file, log_message.take_message().unwrap())
                .unwrap();
        }

        let close_message = LogMessage::log_closed().take_message().unwrap();
        LogCenter::print_log_message(&close_message);
        writer.write_to_file(&mut file, close_message).unwrap();
    }

    fn print_log_message(message: &str) {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => println!("At {}: {}", n.as_secs(), message),
            Err(_) => panic!("SystemTime before UNIX EPOCH! Are we travelling to the past?"),
        }
    }
}

#[cfg(test)]
pub mod test_log_center {

    use super::*;
    use std::fs;
    use std::fs::File;
    use std::sync::mpsc;

    #[test]
    fn test01_sending_a_log_message() {
        let _ = File::create("example1.txt").unwrap();
        let config = Arc::new(Mutex::new(RedisConfig::new(
            String::new(),
            String::new(),
            String::from("example1.txt"),
            0,
        )));
        let writer = FileManager::new();
        let message = LogMessage::test_message1();
        let (sender, receiver) = mpsc::channel();
        let _log_center = LogCenter::new(receiver, config, writer);

        sender.send(message).unwrap();
        thread::sleep(std::time::Duration::from_millis(1));

        assert_eq!(
            fs::read("example1.txt").unwrap(),
            b"$14\r\nThis is test 1\r\n"
        );
    }

    #[test]
    fn test02_sending_a_log_message_and_drop_log_center() {
        {
            let _ = File::create("example2.txt").unwrap();
            let config = Arc::new(Mutex::new(RedisConfig::new(
                String::new(),
                String::new(),
                String::from("example2.txt"),
                0,
            )));
            let writer = FileManager::new();
            let message = LogMessage::test_message2();
            let (sender, receiver) = mpsc::channel();
            let _log_center = LogCenter::new(receiver, config, writer);
            sender.send(message).unwrap();
            thread::sleep(std::time::Duration::from_millis(1));
            assert_eq!(
                fs::read("example2.txt").unwrap(),
                b"$14\r\nThis is test 2\r\n"
            );
        }

        thread::sleep(std::time::Duration::from_millis(1));
        assert_eq!(
            fs::read("example2.txt").unwrap(),
            b"$14\r\nThis is test 2\r\n$21\r\nLog center is closed.\r\n"
        );
    }
}
