use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::SystemTime;
use std::sync::{Arc, Mutex};


use crate::comunication::log_messages::LogMessage;
use crate::native_types::error::ErrorStruct;

pub struct FileHandler {
    last_message: Option<String>
}

impl FileHandler {
    fn new() -> FileHandler{
        FileHandler{
            last_message: None,
        }
    }

    fn write_line(&mut self, line: String) {
        self.last_message = Some(line);
    }

    fn get(&mut self) -> Option<String> {
        self.last_message.take()
    }
}

pub struct LogCenter{
    handler: Option<JoinHandle<()>>
}

impl LogCenter {
    
    pub fn new(receiver: Receiver<LogMessage>, verbose_mode: usize, writer: Arc<Mutex<FileHandler>>) -> Result<LogCenter, ErrorStruct> {
        
        let builder = thread::Builder::new().name("Log Center".into());
        let log_handler = LogCenter::spawn_handler(builder, receiver, verbose_mode, writer)?;

        Ok(LogCenter{
            handler: Some(log_handler),
        })

    }

    fn spawn_handler(builder: thread::Builder, receiver: Receiver<LogMessage>, verbose_mode: usize, writer: Arc<Mutex<FileHandler>>) -> Result<JoinHandle<()>, ErrorStruct> {
        match builder.spawn(move || {LogCenter::start(receiver, verbose_mode, writer)}) {
            Ok(handler) => Ok(handler),
            Err(_) => Err(ErrorStruct::new("INITFAILED".to_string(), "Failed to create Log Center".to_string())),
        }
    }

    fn start(receiver: Receiver<LogMessage>, verbose_mode: usize, writer: Arc<Mutex<FileHandler>>) {
        for mut log_message in receiver.iter() {
            
            if let Some(message) = log_message.is_verbosely_printable(&verbose_mode) {
                LogCenter::print_log_message(message);
            }
            
            match writer.lock(){
                Ok(mut handler) => handler.write_line(log_message.take_message().unwrap()),
                Err(_) => panic!(),
            }

        }

        let close_message = LogMessage::log_closed().take_message().unwrap();
        LogCenter::print_log_message(&close_message);
        writer.lock().unwrap().write_line(close_message);
    }

    fn print_log_message(message: &String) {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => println!("At {}: {}", n.as_secs(), message),
            Err(_) => panic!("SystemTime before UNIX EPOCH! Are we travelling to the past?"),
        }
    }

}

#[cfg(test)]
pub mod test_log_center {

    use super::*;

    #[test]
    fn test01_sending_a_log_message(){
        let mut writer = Arc::new(Mutex::new(FileHandler::new()));
        let mut message = LogMessage::test_message();
        let (sender, receiver) = mpsc::channel();
        let log_center = LogCenter::new(receiver, 5, Arc::clone(&writer));

        sender.send(message);
        thread::sleep(std::time::Duration::from_millis(1));
        assert_eq!(writer.lock().unwrap().get(), Some("This is a test".to_string()));
    }

    #[test]
    fn test01_sending_a_log_message_and_drop_log_center(){
        let mut writer = Arc::new(Mutex::new(FileHandler::new()));
        let writer_clone = Arc::clone(&writer);
        {
            let mut message = LogMessage::test_message();
            let (sender, receiver) = mpsc::channel();
            let log_center = LogCenter::new(receiver, 5, Arc::clone(&writer_clone));
            sender.send(message);
            thread::sleep(std::time::Duration::from_millis(1));
            assert_eq!(writer_clone.lock().unwrap().get(), Some("This is a test".to_string()));
        }

        thread::sleep(std::time::Duration::from_millis(1));
        assert_eq!(writer.lock().unwrap().get(), Some("Log center is closed.".to_string()));

    }

}