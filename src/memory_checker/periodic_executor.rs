use crate::{
    joinable::Joinable,
    messages::redis_messages,
    native_types::error_severity::ErrorSeverity,
    tcp_protocol::{
        client_atributes::client_fields::ClientFields, close_thread, notifier::Notifier,
    },
};

use crate::native_types::ErrorStruct;

use crate::tcp_protocol::Response;

use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// This structure sleeps periodically and execute
/// a given command. When it is needed, the loop
/// stops and ends the periodic execution.
pub struct PeriodicExecutor {
    handle: Option<JoinHandle<Result<(), ErrorStruct>>>,
    still_working: Arc<AtomicBool>,
    notifier: Notifier,
    name: String,
}

impl PeriodicExecutor {
    /// Creates the structure
    pub fn new(
        command: Vec<String>,
        period: u64,
        notifier: Notifier,
        name: &str,
    ) -> PeriodicExecutor {
        let still_working = Arc::new(AtomicBool::new(true));
        let still_working_clone = Arc::clone(&still_working);
        let c_notifier = notifier.clone();

        let garbage_collector_handle = std::thread::spawn(move || {
            PeriodicExecutor::init(command, period, c_notifier, still_working_clone)
        });

        PeriodicExecutor {
            handle: Some(garbage_collector_handle),
            still_working,
            notifier,
            name: String::from(name),
        }
    }

    /// Initialize the loop that periodically send the
    /// command.
    fn init(
        command: Vec<String>,
        period: u64,
        notifier: Notifier,
        still_working_clone: Arc<AtomicBool>,
    ) -> Result<(), ErrorStruct> {
        let (snd_rsp, rcv_rsp): (mpsc::Sender<Response>, mpsc::Receiver<Response>) =
            mpsc::channel();
        let mut counter = 0;

        loop {
            thread::sleep(Duration::new(1, 0));
            counter += 1;

            if !still_working_clone.load(Ordering::Relaxed) {
                return Ok(());
            } else if counter == period {
                counter = 0;
                notifier.send_command_delegator(Some((
                    command.clone(),
                    snd_rsp.clone(),
                    Arc::new(Mutex::new(ClientFields::default())),
                )))?;
                PeriodicExecutor::receive_result(&rcv_rsp)?;
                notifier.notify_successful_shipment(
                    &Arc::new(Mutex::new(ClientFields::default())),
                    command.clone(),
                )?;
            }
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

    fn receive_result(rcv_rsp: &Receiver<Result<String, ErrorStruct>>) -> Result<(), ErrorStruct> {
        PeriodicExecutor::check_severity(
            rcv_rsp
                .recv()
                .map_err(|_| {
                    ErrorStruct::from(redis_messages::closed_sender(ErrorSeverity::ShutdownServer))
                })?
                .map(|_| ()),
        )
    }

    /// Stops the loop and finishes the job
    fn stop(&mut self) {
        self.still_working.store(false, Ordering::Relaxed);
    }
}

impl Joinable<()> for PeriodicExecutor {
    fn join(&mut self) -> Result<(), ErrorStruct> {
        self.stop();
        close_thread(
            self.handle.take(),
            &format!("Periodic Executor ({})", self.name),
            self.notifier.clone(),
        )?;
        Ok(())
    }
}

#[cfg(test)]

mod test_periodic_executor {

    use std::sync::mpsc::{Receiver, Sender};

    use super::*;
    use crate::{
        native_types::{RSimpleString, RedisType},
        tcp_protocol::RawCommand,
    };

    // Para probar los test 1 y 3, hagan fallar el test
    // y verifiquen que se imprima un mensaje indicando que
    // se dropeo el Garbage Collector

    #[test]
    #[ignore = "Long test"]
    fn long_test_01_periodic_executor_is_dropped_safely() {
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

        let command = vec!["clean".to_string(), "20".to_string()];
        let _collector = PeriodicExecutor::new(command, 1, notifier, "clean");

        assert_eq!(4, 4);
    }

    #[test]
    #[ignore = "Long test"]
    fn long_test_02_garbage_collector_send_the_correct_command() {
        let (snd_test_cmd, rcv_test_cmd) = mpsc::channel();

        let (snd_log_test, _) = mpsc::channel();

        let notifier = Notifier::new(
            snd_log_test.clone(),
            snd_test_cmd.clone(),
            Arc::new(AtomicBool::new(false)),
            "test_addr".into(),
        );
        let command = vec!["clean".to_string(), "20".to_string()];
        let _collector = PeriodicExecutor::new(command, 1, notifier, "clean");
        let (command_recv, sender, _): RawCommand = rcv_test_cmd.recv().unwrap().unwrap();

        assert_eq!(&command_recv[0], "clean");
        assert_eq!(&command_recv[1], "20");
        sender
            .send(Ok(RSimpleString::encode("OK".to_string())))
            .unwrap();
    }

    #[test]
    #[ignore = "Long test"]
    fn long_test_03_returning_an_error_drops_the_garbage_collector() {
        let (snd_test_cmd, rcv_test_cmd) = mpsc::channel();

        let (snd_log_test, _) = mpsc::channel();

        let notifier = Notifier::new(
            snd_log_test.clone(),
            snd_test_cmd.clone(),
            Arc::new(AtomicBool::new(false)),
            "test_addr".into(),
        );
        let command = vec!["clean".to_string(), "20".to_string()];
        let _collector = PeriodicExecutor::new(command, 1, notifier, "clean");
        let (_command, sender, _) = rcv_test_cmd.recv().unwrap().unwrap();
        sender
            .send(Err(ErrorStruct::new(
                "ERR".to_string(),
                "this is a generic error".to_string(),
            )))
            .unwrap();

        assert_eq!(4, 4);
    }
}
