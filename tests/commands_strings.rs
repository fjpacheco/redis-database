use redis::{Commands, RedisError};
use redis_rust::native_types::ErrorStruct;

// https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html
// importing setup module.
mod setup;

#[test]
#[ignore = "Integration Test"]
fn int_test_01_set_key_with_value_return_ok() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;

    let received: Result<String, RedisError> = connection_client.set("key", "value");

    assert!(received.is_ok());
    assert_eq!(received, Ok("OK".to_string()));
    server.shutdown()?;
    Ok(())
}

#[test]
#[ignore = "Integration Test"]
fn int_test_02_get_return_a_value() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key", "value");

    let received: Result<String, RedisError> = connection_client.get("key");

    assert!(received.is_ok());
    assert_eq!(received, Ok("value".to_string()));
    server.shutdown()?;
    Ok(())
}
//}
