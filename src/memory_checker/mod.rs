use std::sync::mpsc::Sender;

pub mod garbage_collector;

type RawCommand = (Vec<String>, Sender<String>);
