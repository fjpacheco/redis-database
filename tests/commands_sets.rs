use redis::{Commands, RedisError};
use redis_rust::native_types::ErrorStruct;

// https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html
// importing setup module.
mod setup;

#[test]
#[ignore = "Integration Test"]
fn int_test_01_sadd() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value");
    let _: Result<String, RedisError> = connection_client.lpush("key_2", vec!["item_1", "item_2"]);

    let received_1: Result<isize, RedisError> =
        connection_client.sadd("key_1", vec!["member_1", "member_2"]);

    let received_2: Result<isize, RedisError> =
        connection_client.sadd("key_2", vec!["member_1", "member_2"]);

    let received_3: Result<isize, RedisError> =
        connection_client.sadd("key_3", vec!["member_1", "member_2", "member_3"]);

    assert!(received_1.is_err());
    assert!(received_2.is_err());
    assert_eq!(received_3, Ok(3));

    server.shutdown()?;
    Ok(())
}

#[test]
#[ignore = "Integration Test"]
fn int_test_02_scard() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value");
    let _: Result<String, RedisError> = connection_client.lpush("key_2", vec!["item_1", "item_2"]);
    let _: Result<isize, RedisError> =
        connection_client.sadd("key_3", vec!["member_1", "member_2"]);
    let _: Result<isize, RedisError> = connection_client.sadd(
        "key_4",
        vec!["member_1", "member_2", "member_3", "member_4"],
    );

    let received_1: Result<isize, RedisError> = connection_client.scard("key_1");
    let received_2: Result<isize, RedisError> = connection_client.scard("key_2");
    let received_3: Result<isize, RedisError> = connection_client.scard("key_3");
    let received_4: Result<isize, RedisError> = connection_client.scard("key_4");

    assert!(received_1.is_err());
    assert!(received_2.is_err());
    assert_eq!(received_3, Ok(2));
    assert_eq!(received_4, Ok(4));

    server.shutdown()?;
    Ok(())
}

#[test]
#[ignore = "Integration Test"]
fn int_test_03_srem() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value");
    let _: Result<String, RedisError> = connection_client.lpush("key_2", vec!["item_1", "item_2"]);
    let _: Result<isize, RedisError> =
        connection_client.sadd("key_3", vec!["member_1", "member_2"]);
    let _: Result<isize, RedisError> = connection_client.sadd(
        "key_4",
        vec!["member_1", "member_2", "member_3", "member_4"],
    );

    let received_1: Result<isize, RedisError> = connection_client.srem("key_1", vec!["value"]);
    let received_2: Result<isize, RedisError> =
        connection_client.srem("key_2", vec!["item_1", "item_2"]);
    let received_3: Result<isize, RedisError> = connection_client.srem("key_3", vec!["member_1"]);
    let received_4: Result<isize, RedisError> =
        connection_client.srem("key_4", vec!["member_2", "member_3", "member_4"]);
    assert!(received_1.is_err());
    assert!(received_2.is_err());
    assert_eq!(received_3, Ok(1));
    assert_eq!(received_4, Ok(3));

    let received_3_scard: Result<isize, RedisError> = connection_client.scard("key_3");
    let received_4_scard: Result<isize, RedisError> = connection_client.scard("key_4");

    assert_eq!(received_3_scard, Ok(1));
    assert_eq!(received_4_scard, Ok(1));

    server.shutdown()?;
    Ok(())
}

#[test]
#[ignore = "Integration Test"]
fn int_test_04_smembers() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value");
    let _: Result<String, RedisError> = connection_client.lpush("key_2", vec!["item_1", "item_2"]);
    let _: Result<isize, RedisError> =
        connection_client.sadd("key_3", vec!["member_1", "member_2"]);
    let _: Result<isize, RedisError> = connection_client.sadd(
        "key_4",
        vec!["member_1", "member_2", "member_3", "member_4"],
    );

    let received_1: Result<Vec<String>, RedisError> = connection_client.smembers("key_1");
    let received_2: Result<Vec<String>, RedisError> = connection_client.smembers("key_2");
    let received_3: Result<Vec<String>, RedisError> = connection_client.smembers("key_3");
    let received_4: Result<Vec<String>, RedisError> = connection_client.smembers("key_4");
    assert!(received_1.is_err());
    assert!(received_2.is_err());

    let vector_received_3 = received_3.unwrap();
    let vector_received_4 = received_4.unwrap();
    assert!(vector_received_3.contains(&String::from("member_1")));
    assert!(vector_received_3.contains(&String::from("member_2")));
    assert!(vector_received_4.contains(&String::from("member_1")));
    assert!(vector_received_4.contains(&String::from("member_2")));
    assert!(vector_received_4.contains(&String::from("member_3")));
    assert!(vector_received_4.contains(&String::from("member_4")));

    server.shutdown()?;
    Ok(())
}

#[test]
#[ignore = "Integration Test"]
fn int_test_05_sismember() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value");
    let _: Result<String, RedisError> = connection_client.lpush("key_2", vec!["item_1", "item_2"]);
    let _: Result<isize, RedisError> =
        connection_client.sadd("key_3", vec!["member_1", "member_2"]);
    let _: Result<isize, RedisError> = connection_client.sadd(
        "key_4",
        vec!["member_1", "member_2", "member_3", "member_4"],
    );

    let received_1: Result<isize, RedisError> = connection_client.sismember("key_1", "value");
    let received_2: Result<isize, RedisError> = connection_client.sismember("key_2", "item_2");
    let received_3: Result<isize, RedisError> = connection_client.sismember("key_3", "member_1");
    let received_4: Result<isize, RedisError> = connection_client.sismember("key_3", "member_2");
    let received_5: Result<isize, RedisError> = connection_client.sismember("key_3", "member_3");
    let received_6: Result<isize, RedisError> = connection_client.sismember("key_4", "member_1");
    let received_7: Result<isize, RedisError> = connection_client.sismember("key_4", "member_2");
    let received_8: Result<isize, RedisError> = connection_client.sismember("key_4", "member_3");
    let received_9: Result<isize, RedisError> = connection_client.sismember("key_4", "member_4");
    let received_10: Result<isize, RedisError> = connection_client.sismember("key_4", "member_5");

    assert!(received_1.is_err());
    assert!(received_2.is_err());
    assert_eq!(received_3, Ok(1));
    assert_eq!(received_4, Ok(1));
    assert_eq!(received_5, Ok(0));
    assert_eq!(received_6, Ok(1));
    assert_eq!(received_7, Ok(1));
    assert_eq!(received_8, Ok(1));
    assert_eq!(received_9, Ok(1));
    assert_eq!(received_10, Ok(0));

    server.shutdown()?;
    Ok(())
}
