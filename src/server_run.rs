use std::env;

use redis_rust::{native_types::ErrorStruct, tcp_protocol::server::ServerRedis};

/// ## Commands to Start Server, in console:
///
/// With configs default in cargo:
///
/// * *cargo run*
///
/// With configs personalized in cargo
///
/// * *cargo run /path/to/redis.conf*
///
/// With the executable generated with *cargo build*
///
/// * With config default
///     * *target/debug/server*
/// * With config personalized:
///     * *target/debug/server /path/to/redis.conf*
/// * To execute server visualizing the Life of Threads with *gdb*
///     * *rust-gdb target/debug/server*
///
fn main() -> Result<(), ErrorStruct> {
    let argv: Vec<String> = env::args().collect();
    ServerRedis::start(argv)?;
    Ok(())
}

/*
fn main() {
    for line in stdin().lock().lines() {
        match line {
            Ok(line) => process(&*line),
            Err(e) => panic!("{}", e),
        }
    }
}*/
