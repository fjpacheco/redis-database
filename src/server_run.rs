use std::env;

use redis_rust::{native_types::ErrorStruct, tcp_protocol::server::ServerRedis};

/// ## Start a server
///
/// Commands to start server:
///
/// With configs default:
///
/// * *cargo run*
///
/// With configs personalized
///
/// * *cargo run /path/to/redis.conf*
///
fn main() -> Result<(), ErrorStruct> {
    let argv: Vec<String> = env::args().collect();
    let _server = ServerRedis::start(argv)?;
    Ok(())
}
