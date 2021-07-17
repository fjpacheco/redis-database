use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::SystemTime;

use crate::communication::log_messages::LogMessage;
use crate::file_manager::FileManager;
use crate::joinable::Joinable;
use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
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
    handler: Option<JoinHandle<Result<(), ErrorStruct>>>,
    sender_log: Sender<Option<LogMessage>>,
    redis_config: Arc<Mutex<RedisConfig>>,
}

impl Joinable<()> for LogCenter {
    fn join(&mut self) -> Result<(), ErrorStruct> {
        println!("LOG CENTER: PODER DECIR ADIÓS ES CRECER");

        let _ = self.sender_log.send(None);

        /*match self.sender.send(None) {
            Ok(()) => { /* Log Center has been closed right now */ }
            Err(_) => { /* Log Center is already closed */ }
        }*/

        if let Some(handle) = self.handler.take() {
            match handle.join() {
                Ok(result) => self.notify_result(result)?,
                Err(_) => self.notify_result(Err(ErrorStruct::from(
                    redis_messages::thread_panic("log center"),
                )))?,
            }
        }
        Ok(())
    }
}

impl LogCenter {
    pub fn new(
        sender: Sender<Option<LogMessage>>,
        receiver: Receiver<Option<LogMessage>>,
        redis_config: Arc<Mutex<RedisConfig>>,
        writer: FileManager,
    ) -> Result<LogCenter, ErrorStruct> {
        let builder = thread::Builder::new().name("Log Center".into());
        let log_handler =
            LogCenter::spawn_handler(builder, receiver, Arc::clone(&redis_config), writer)?;

        Ok(LogCenter {
            handler: Some(log_handler),
            sender_log: sender,
            redis_config,
        })
    }

    fn spawn_handler(
        builder: thread::Builder,
        receiver: Receiver<Option<LogMessage>>,
        redis_config: Arc<Mutex<RedisConfig>>,
        writer: FileManager,
    ) -> Result<JoinHandle<Result<(), ErrorStruct>>, ErrorStruct> {
        match builder.spawn(move || LogCenter::start(receiver, redis_config, writer)) {
            Ok(handler) => Ok(handler),
            Err(_) => Err(ErrorStruct::from(redis_messages::init_failed(
                "Log Center",
                ErrorSeverity::ShutdownServer,
            ))),
        }
    }

    fn start(
        receiver: Receiver<Option<LogMessage>>,
        redis_config: Arc<Mutex<RedisConfig>>,
        writer: FileManager,
    ) -> Result<(), ErrorStruct> {
        for packed_log_message in receiver.iter() {
            if let Some(log_message) = packed_log_message {
                LogCenter::print_log_message(&log_message, Arc::clone(&redis_config))?;
                LogCenter::write_log(log_message, &writer, Arc::clone(&redis_config))?;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn print_log_message(
        log_message: &LogMessage,
        redis_config: Arc<Mutex<RedisConfig>>,
    ) -> Result<(), ErrorStruct> {
        let mutexguard_verbose = redis_config.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "redis_config",
                ErrorSeverity::ShutdownServer,
            ))
        })?;

        if let Some(message) = log_message.is_verbosely_printable(mutexguard_verbose.verbose()) {
            match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => println!("At {}: {}", n.as_secs(), message),
                Err(_) => {
                    return Err(ErrorStruct::from(redis_messages::unexpected_behaviour(
                        "SystemTime before UNIX EPOCH! Are we travelling to the past?",
                    )));
                }
            }
        }

        Ok(())
    }

    fn write_log(
        mut log_message: LogMessage,
        writer: &FileManager,
        redis_config: Arc<Mutex<RedisConfig>>,
    ) -> Result<(), ErrorStruct> {
        let mut mutexguard_line_writer = redis_config.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "redis_config",
                ErrorSeverity::ShutdownServer,
            ))
        })?;

        if let Some(file) = mutexguard_line_writer.get_mut_linewriter() {
            writer.write_to_file(file, log_message.take_message().unwrap())?;
        }
        Ok(())
    }

    fn notify_result(&mut self, result: Result<(), ErrorStruct>) -> Result<(), ErrorStruct> {
        if let Err(error) = result {
            self.print_and_write_notification(LogMessage::from_errorstruct(error))?;
        }

        self.print_and_write_notification(LogMessage::log_closed_success())?;
        Ok(())
    }

    fn print_and_write_notification(&self, close_message: LogMessage) -> Result<(), ErrorStruct> {
        LogCenter::print_log_message(&close_message, self.redis_config.clone())?;
        LogCenter::write_log(
            close_message,
            &FileManager::new(),
            self.redis_config.clone(),
        )?;
        Ok(())
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
        let mut log_center = LogCenter::new(sender.clone(), receiver, config, writer).unwrap();

        sender.send(Some(message)).unwrap();
        thread::sleep(std::time::Duration::from_millis(1));
        assert_eq!(
            fs::read("example4.txt").unwrap(),
            b"$14\r\nThis is test 1\r\n"
        );
        drop(sender);
        log_center.join().unwrap();
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
            let mut log_center = LogCenter::new(sender.clone(), receiver, config, writer).unwrap();
            sender.send(Some(message)).unwrap();
            thread::sleep(std::time::Duration::from_millis(1));
            assert_eq!(
                fs::read("example5.txt").unwrap(),
                b"$14\r\nThis is test 2\r\n"
            );
            drop(sender);
            log_center.join().unwrap();
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
        let mut log_center =
            LogCenter::new(sender.clone(), receiver, Arc::clone(&config), writer).unwrap();

        sender.send(Some(message1)).unwrap();
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
        sender.send(Some(message2)).unwrap();
        thread::sleep(std::time::Duration::from_millis(1)); // Para no leer el archivo antes de que se escriba

        assert_eq!(
            fs::read("new_file_name.txt").unwrap(),
            b"$14\r\nThis is test 1\r\n"
        );
        drop(sender);
        log_center.join().unwrap();
    }
}
