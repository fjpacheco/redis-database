use std::usize;

use redis::{Commands, RedisError};
use redis_rust::native_types::ErrorStruct;

// https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html
// importing setup module.
mod setup;

#[test]
#[ignore = "Integration Test"]
fn int_test_01_sadd_key_with_members_return_ok() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;

    let received: Result<usize, RedisError> =
        connection_client.sadd("key", vec!["member1", "member2"]);

    assert!(received.is_ok());
    assert_eq!(received, Ok(2));
    server.shutdown()?;
    Ok(())
}
