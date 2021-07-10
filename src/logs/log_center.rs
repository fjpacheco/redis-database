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
    handler: Option<JoinHandle<()>>,
}
impl Drop for LogCenter {
    fn drop(&mut self) {
        println!("LOG CENTER: PODER DECIR ADIÓS ES CRECER");
        if let Some(handle) = self.handler.take() {
            handle.join().unwrap();
        }
    }
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
            handler: Some(log_handler),
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
        // Considerar caso en el que cambia el archivo redis config
        for mut log_message in receiver.iter() {
            if let Some(message) =
                log_message.is_verbosely_printable(redis_config.lock().unwrap().verbose())
            {
                LogCenter::print_log_message(message);
            }
            if let Some(file) = redis_config.lock().unwrap().get_mut_linewriter() {
                writer
                    .write_to_file(file, log_message.take_message().unwrap())
                    .unwrap();
                //TODO: manejar error de no escritura, cortar todo xD
            }
        }

        let close_message = LogMessage::log_closed().take_message().unwrap();
        LogCenter::print_log_message(&close_message);
        if let Some(mut file) = redis_config.lock().unwrap().get_mut_linewriter() {
            writer.write_to_file(&mut file, close_message).unwrap();
            //TODO: manejar error de no escritura, cortar todo xD
        }
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
    use std::{fs::File, sync::mpsc};

    #[test]
    fn test01_sending_a_log_message() {
        let _ = File::create("example4.txt").unwrap();
        let config = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("example4.txt"),
                0,
            )
            .unwrap(),
        ));
        let writer = FileManager::new();
        let message = LogMessage::test_message1();
        let (sender, receiver) = mpsc::channel();
        let _log_center = LogCenter::new(receiver, config, writer);

        sender.send(message).unwrap();
        thread::sleep(std::time::Duration::from_millis(1));
        assert_eq!(
            fs::read("example4.txt").unwrap(),
            b"$14\r\nThis is test 1\r\n"
        );
        drop(sender);
    }

    #[test]
    fn test02_sending_a_log_message_and_drop_log_center() {
        {
            let _ = File::create("example5.txt").unwrap();
            let config = Arc::new(Mutex::new(
                RedisConfig::new(
                    String::new(),
                    String::new(),
                    String::from("example5.txt"),
                    0,
                )
                .unwrap(),
            ));
            let writer = FileManager::new();
            let message = LogMessage::test_message2();
            let (sender, receiver) = mpsc::channel();
            let _log_center = LogCenter::new(receiver, config, writer);
            sender.send(message).unwrap();
            thread::sleep(std::time::Duration::from_millis(1));
            assert_eq!(
                fs::read("example5.txt").unwrap(),
                b"$14\r\nThis is test 2\r\n"
            );
            drop(sender)
        }

        thread::sleep(std::time::Duration::from_millis(1));
        assert_eq!(
            fs::read("example5.txt").unwrap(),
            b"$14\r\nThis is test 2\r\n$21\r\nLog center is closed.\r\n"
        );
    }

    #[test]
    fn test03_changing_logfile_name() {
        let _ = File::create("example6.txt").unwrap();
        let config = Arc::new(Mutex::new(
            RedisConfig::new(
                String::new(),
                String::new(),
                String::from("example6.txt"),
                0,
            )
            .unwrap(),
        ));
        let writer = FileManager::new();
        let message1 = LogMessage::test_message1();
        let (sender, receiver) = mpsc::channel();
        let _log_center = LogCenter::new(receiver, Arc::clone(&config), writer);

        sender.send(message1).unwrap();
        thread::sleep(std::time::Duration::from_millis(1));

        assert_eq!(
            fs::read("example6.txt").unwrap(),
            b"$14\r\nThis is test 1\r\n"
        );

        let _ = File::create("new_file_name.txt").unwrap();
        config
            .lock()
            .unwrap()
            .change_file("new_file_name.txt".to_string())
            .unwrap();

        let message2 = LogMessage::test_message1();
        sender.send(message2).unwrap();
        thread::sleep(std::time::Duration::from_millis(1)); // Para no leer el archivo antes de que se escriba

        assert_eq!(
            fs::read("new_file_name.txt").unwrap(),
            b"$14\r\nThis is test 1\r\n"
        );
        drop(sender)
    }
}
