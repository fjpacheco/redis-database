use crate::memory_checker::RawCommand;
use crate::native_types::redis_type::RedisType;
use crate::native_types::ErrorStruct;
use crate::native_types::RError;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct GarbageCollector {
    handle: Option<JoinHandle<()>>,
    still_working: Arc<AtomicBool>,
}

impl GarbageCollector {
    pub fn start(
        snd_to_dat_del: mpsc::Sender<RawCommand>,
        period: u64,
        keys_touched: u64,
    ) -> GarbageCollector {
        let (snd_rsp, rcv_rsp): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
        let still_working = Arc::new(AtomicBool::new(true));
        let still_working_clone = Arc::clone(&still_working);

        let garbage_collector_handle = std::thread::spawn(move || {
            loop {
                thread::sleep(Duration::new(period, 0));

                if !still_working_clone.load(Ordering::Relaxed) {
                    //log
                    println!("Shutting down Garbage Collector");
                    break;
                }

                let command = vec!["clean".to_string(), keys_touched.to_string()];
                snd_to_dat_del.send((command, snd_rsp.clone())).unwrap();

                match rcv_rsp.recv() {
                    Ok(response) => match is_err(response) {
                        Ok(()) => { /*log*/ }
                        Err(_) => {
                            //log
                            break;
                        }
                    },
                    Err(_) => {
                        //log
                        break;
                    }
                }
            }
        });

        GarbageCollector {
            handle: Some(garbage_collector_handle),
            still_working,
        }
    }

    fn stop(&mut self) {
        self.still_working.store(false, Ordering::Relaxed);
    }
}

impl Drop for GarbageCollector {
    fn drop(&mut self) {
        self.stop();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
        println!("Garbage collector has been shutted down!");
    }
}

fn is_err(response: String) -> Result<(), ErrorStruct> {
    let mut buffer = BufReader::new(response.as_bytes());
    let mut first_lecture = String::new();
    buffer.read_line(&mut first_lecture).unwrap();
    let redis_type = first_lecture.remove(0); // Redis Type inference
    first_lecture.pop().unwrap(); // popping \n
    first_lecture.pop().unwrap(); // popping \r
    match redis_type {
        '+' => Ok(()),
        '-' => Err(RError::decode(first_lecture, &mut buffer.lines()).unwrap()),
        _ => Err(ErrorStruct::new(
            "ERR".to_string(),
            "something went wrong in garbage collector".to_string(),
        )),
    }
}

#[cfg(test)]

mod test_garbage_collector {

    use super::*;
    use crate::native_types::RSimpleString;

    // Para probar los test 1 y 3, hagan fallar el test
    // y verifiquen que se imprima un mensaje indicando que
    // se dropeo el Garbage Collector

    #[test]
    #[ignore]
    fn test01_garbage_collector_is_dropped_safely() {
        let (snd_col_test, _rcv_col_test): (mpsc::Sender<RawCommand>, mpsc::Receiver<RawCommand>) =
            mpsc::channel();
        let _collector = GarbageCollector::start(snd_col_test, 4, 20);

        assert_eq!(4, 4);
    }

    #[test]
    #[ignore]
    fn test02_garbage_collector_send_the_correct_command() {
        let (snd_col_test, rcv_col_test): (mpsc::Sender<RawCommand>, mpsc::Receiver<RawCommand>) =
            mpsc::channel();
        let _collector = GarbageCollector::start(snd_col_test, 4, 20);
        let (command, sender) = rcv_col_test.recv().unwrap();
        assert_eq!(&command[0], "clean");
        assert_eq!(&command[1], "20");
        sender
            .send(RSimpleString::encode("OK".to_string()))
            .unwrap();
    }

    #[test]
    #[ignore]
    fn test03_returning_an_error_drops_the_garbage_collector() {
        let (snd_col_test, rcv_col_test): (mpsc::Sender<RawCommand>, mpsc::Receiver<RawCommand>) =
            mpsc::channel();
        let _collector = GarbageCollector::start(snd_col_test, 4, 20);
        let (_command, sender) = rcv_col_test.recv().unwrap();
        sender
            .send(RError::encode(ErrorStruct::new(
                "ERR".to_string(),
                "this is a generic error".to_string(),
            )))
            .unwrap();

        assert_eq!(4, 4);
    }
}
