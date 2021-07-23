use crate::{
    joinable::Joinable,
    messages::redis_messages,
    native_types::error_severity::ErrorSeverity,
    tcp_protocol::{
        client_atributes::client_fields::ClientFields, close_thread, notifier::Notifier, RawCommand,
    },
};

use crate::native_types::ErrorStruct;

use crate::tcp_protocol::Response;

use std::sync::mpsc;
use std::sync::Arc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct GarbageCollector {
    handle: Option<JoinHandle<Result<(), ErrorStruct>>>,
    still_working: Arc<AtomicBool>,
    notifier: Notifier,
}

/// Garbage Collector cleans periodically some keys of
/// the database which are expired. The period and the
/// number of keys that would be checked are customizable.

impl GarbageCollector {
    /// Creates the structure
    pub fn new(
        snd_to_dat_del: mpsc::Sender<Option<RawCommand>>,
        period: u64,
        keys_touched: u64,
        notifier: Notifier,
    ) -> GarbageCollector {
        let still_working = Arc::new(AtomicBool::new(true));
        let still_working_clone = Arc::clone(&still_working);

        let garbage_collector_handle = std::thread::spawn(move || {
            GarbageCollector::init(snd_to_dat_del, period, keys_touched, still_working_clone)
        });

        GarbageCollector {
            handle: Some(garbage_collector_handle),
            still_working,
            notifier,
        }
    }

    /// Initialize the loop that periodically send the
    /// command CLEAN.
    fn init(
        snd_to_dat_del: mpsc::Sender<Option<RawCommand>>,
        period: u64,
        keys_touched: u64,
        still_working_clone: Arc<AtomicBool>,
    ) -> Result<(), ErrorStruct> {
        let (snd_rsp, rcv_rsp): (mpsc::Sender<Response>, mpsc::Receiver<Response>) =
            mpsc::channel();
        loop {
            thread::sleep(Duration::new(period, 0));

            if !still_working_clone.load(Ordering::Relaxed) {
                return Ok(());
            }

            let command = vec!["clean".to_string(), keys_touched.to_string()];
            snd_to_dat_del
                .send(Some((
                    command,
                    snd_rsp.clone(),
                    Arc::new(Mutex::new(ClientFields::default())),
                )))
                .map_err(|_| {
                    ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::ShutdownServer))
                })?;

            GarbageCollector::check_severity(
                rcv_rsp
                    .recv()
                    .map_err(|_| {
                        ErrorStruct::from(redis_messages::closed_sender(
                            ErrorSeverity::ShutdownServer,
                        ))
                    })?
                    .map(|_| ()),
            )?;
        }
    }

    fn check_severity(packed_error: Result<(), ErrorStruct>) -> Result<(), ErrorStruct> {
        if let Err(error) = packed_error {
            if let Some(ErrorSeverity::ShutdownServer) = error.severity() {
                return Err(error);
            }
        }
        Ok(())
    }

    /// Stops the loop and finishes the job
    fn stop(&mut self) {
        self.still_working.store(false, Ordering::Relaxed);
    }
}

impl Joinable<()> for GarbageCollector {
    fn join(&mut self) -> Result<(), ErrorStruct> {
        println!("🥽🥽🥽🥽🥽");
        self.stop();
        println!("🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽🥽");
        close_thread(
            self.handle.take(),
            "Garbage collector",
            self.notifier.clone(),
        )?;
        println!("🥽🥽🥽🥽🥽🥽Garbage collector has been shutted down!");
        Ok(())
    }
}

#[cfg(test)]

mod test_garbage_collector {

    use std::sync::mpsc::{Receiver, Sender};

    use super::*;
    use crate::native_types::{RSimpleString, RedisType};

    // Para probar los test 1 y 3, hagan fallar el test
    // y verifiquen que se imprima un mensaje indicando que
    // se dropeo el Garbage Collector

    #[test]
    #[ignore = "Long test"]
    fn long_test_01_garbage_collector_is_dropped_safely() {
        let (snd_col_test, _rcv_col_test) = mpsc::channel();

        let (snd_test_cmd, _rcv_test_cmd): (
            Sender<Option<RawCommand>>,
            Receiver<Option<RawCommand>>,
        ) = mpsc::channel();

        let (snd_log_test, _) = mpsc::channel();

        let notifier = Notifier::new(
            snd_log_test.clone(),
            snd_test_cmd.clone(),
            Arc::new(AtomicBool::new(false)),
            "test_addr".into(),
        );
        let _collector = GarbageCollector::new(snd_col_test, 1, 20, notifier);

        assert_eq!(4, 4);
    }

    #[test]
    #[ignore = "Long test"]
    fn long_test_02_garbage_collector_send_the_correct_command() {
        let (snd_col_test, rcv_col_test) = mpsc::channel();

        let (snd_test_cmd, _rcv_test_cmd) = mpsc::channel();

        let (snd_log_test, _) = mpsc::channel();

        let notifier = Notifier::new(
            snd_log_test.clone(),
            snd_test_cmd.clone(),
            Arc::new(AtomicBool::new(false)),
            "test_addr".into(),
        );
        let _collector = GarbageCollector::new(snd_col_test, 1, 20, notifier);
        let (command, sender, _) = rcv_col_test.recv().unwrap().unwrap();

        assert_eq!(&command[0], "clean");
        assert_eq!(&command[1], "20");
        sender
            .send(Ok(RSimpleString::encode("OK".to_string())))
            .unwrap();
    }

    #[test]
    #[ignore = "Long test"]
    fn long_test_03_returning_an_error_drops_the_garbage_collector() {
        let (snd_col_test, rcv_col_test) = mpsc::channel();
        let (snd_test_cmd, _rcv_test_cmd) = mpsc::channel();

        let (snd_log_test, _) = mpsc::channel();

        let notifier = Notifier::new(
            snd_log_test.clone(),
            snd_test_cmd.clone(),
            Arc::new(AtomicBool::new(false)),
            "test_addr".into(),
        );
        let _collector = GarbageCollector::new(snd_col_test, 1, 20, notifier);
        let (_command, sender, _) = rcv_col_test.recv().unwrap().unwrap();
        sender
            .send(Err(ErrorStruct::new(
                "ERR".to_string(),
                "this is a generic error".to_string(),
            )))
            .unwrap();

        assert_eq!(4, 4);
    }
}
