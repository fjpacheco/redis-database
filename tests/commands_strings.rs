use std::usize;

use redis::{Commands, RedisError};
use redis_rust::{native_types::ErrorStruct, vec_strings};

// https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html
// importing setup module.
mod setup;

#[test]
#[ignore = "Integration Test"]
fn int_test_01_set() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;

    let received_1: Result<String, RedisError> = connection_client.set("key_1", "value");
    let received_2: Result<String, RedisError> = connection_client.set("key_2", "1");
    let received_3: Result<String, RedisError> = connection_client.set("key_4", 2);
    assert_eq!(received_1, Ok("OK".to_string()));
    assert_eq!(received_2, Ok("OK".to_string()));
    assert_eq!(received_3, Ok("OK".to_string()));

    server.shutdown()?;
    Ok(())
}

#[test]
#[ignore = "Integration Test"]
fn int_test_02_get() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value");
    let _: Result<String, RedisError> = connection_client.set("key_2", "1");
    let _: Result<String, RedisError> = connection_client.set("key_3", 2);

    let received_1: Result<String, RedisError> = connection_client.get("key_1");
    let received_2: Result<String, RedisError> = connection_client.get("key_2");
    let received_3: Result<String, RedisError> = connection_client.get("key_3");
    let received_4: Result<String, RedisError> = connection_client.get("key_4");
    assert_eq!(received_1, Ok("value".to_string()));
    assert_eq!(received_2, Ok("1".to_string()));
    assert_eq!(received_3, Ok("2".to_string()));
    assert!(received_4.is_err());

    server.shutdown()?;
    Ok(())
}

#[test]
#[ignore = "Integration Test"]
fn int_test_03_strlen() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "four");
    let _: Result<String, RedisError> = connection_client.set("key_2", "five_");
    let _: Result<String, RedisError> = connection_client.lpush("key_4", vec!["item_1", "item_2"]);
    let _: Result<String, RedisError> =
        connection_client.sadd("key_4", vec!["member_1", "member_2"]);

    let received_1: Result<usize, RedisError> = connection_client.strlen("key_1");
    let received_2: Result<usize, RedisError> = connection_client.strlen("key_2");
    let received_3: Result<usize, RedisError> = connection_client.strlen("key_4");
    let received_4: Result<usize, RedisError> = connection_client.strlen("key_4");
    assert_eq!(received_1, Ok(4));
    assert_eq!(received_2, Ok(5));
    assert!(received_3.is_err());
    assert!(received_4.is_err());

    server.shutdown()?;
    Ok(())
}

#[ignore = "Integration Test"]
#[test]
fn int_test_04_append() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "hello, ");
    let _: Result<String, RedisError> = connection_client.set("key_2", "Rust-");
    let _: Result<String, RedisError> = connection_client.lpush("key_4", vec!["item_1", "item_2"]);
    let _: Result<String, RedisError> =
        connection_client.sadd("key_4", vec!["member_1", "member_2"]);

    let received_1: Result<usize, RedisError> = connection_client.append("key_1", "world");
    let received_2: Result<usize, RedisError> = connection_client.append("key_2", "eze team");
    let received_3: Result<usize, RedisError> = connection_client.append("key_4", "item_3");
    let received_4: Result<usize, RedisError> = connection_client.append("key_4", "item_3");
    let received_5: Result<usize, RedisError> = connection_client.append("key_5", "new_no_setted");
    assert_eq!(received_1, Ok(12));
    assert_eq!(received_2, Ok(13));
    assert!(received_3.is_err());
    assert!(received_4.is_err());
    assert_eq!(received_5, Ok(13));

    let received_1_get: Result<String, RedisError> = connection_client.get("key_1");
    let received_2_get: Result<String, RedisError> = connection_client.get("key_2");
    let received_5_get: Result<String, RedisError> = connection_client.get("key_5");
    assert_eq!(received_1_get, Ok("hello, world".to_string()));
    assert_eq!(received_2_get, Ok("Rust-eze team".to_string()));
    assert_eq!(received_5_get, Ok("new_no_setted".to_string()));

    server.shutdown()?;
    Ok(())
}

#[ignore = "Integration Test"]
#[test]
fn int_test_05_mset() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value_old");
    let _: Result<String, RedisError> = connection_client.lpush("key_3", vec!["item_1", "item_2"]);
    let _: Result<String, RedisError> =
        connection_client.sadd("key_4", vec!["member_1", "member_2"]);

    let received_1: Result<String, RedisError> = redis::cmd("mset")
        .arg("key_1")
        .arg("value_1")
        .arg("key_2")
        .arg("value_2")
        .query(&mut connection_client);

    let received_2: Result<String, RedisError> = redis::cmd("mset")
        .arg("key_3")
        .arg("item_3")
        .query(&mut connection_client);

    let received_3: Result<String, RedisError> = redis::cmd("mset")
        .arg("key_4")
        .arg("member_3")
        .query(&mut connection_client);

    assert_eq!(received_1, Ok("OK".to_string()));
    assert_eq!(received_2, Ok("OK".to_string()));
    assert_eq!(received_3, Ok("OK".to_string()));

    let received_1_get: Result<String, RedisError> = connection_client.get("key_1");
    let received_2_get: Result<String, RedisError> = connection_client.get("key_2");
    let received_3_get: Result<String, RedisError> = connection_client.get("key_3");
    let received_4_get: Result<String, RedisError> = connection_client.get("key_4");
    assert_eq!(received_1_get, Ok("value_1".to_string()));
    assert_eq!(received_2_get, Ok("value_2".to_string()));
    assert_eq!(received_3_get, Ok("item_3".to_string()));
    assert_eq!(received_4_get, Ok("member_3".to_string()));

    server.shutdown()?;
    Ok(())
}

#[ignore = "Integration Test"]
#[test]
fn int_test_06_mget() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value_1");
    let _: Result<String, RedisError> = connection_client.set("key_2", "value_2");
    let _: Result<String, RedisError> = connection_client.lpush("key_3", vec!["item_1", "item_2"]);
    let _: Result<String, RedisError> =
        connection_client.sadd("key_4", vec!["member_1", "member_2"]);

    let received: Result<Vec<String>, RedisError> = redis::cmd("mget")
        .arg("key_1")
        .arg("key_2")
        .arg("key_3")
        .arg("key_4")
        .query(&mut connection_client);
    assert_eq!(received, Ok(vec_strings!("value_1", "value_2")));

    server.shutdown()?;
    Ok(())
}

#[ignore = "Integration Test"]
#[test]
fn int_test_07_incrby() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "10");
    let _: Result<String, RedisError> = connection_client.set("key_2", "0");
    let _: Result<String, RedisError> = connection_client.set("key_3", "-10");
    let _: Result<String, RedisError> = connection_client.set("key_4", "value");
    let _: Result<String, RedisError> = connection_client.lpush("key_5", vec!["item_1", "item_2"]);
    let _: Result<String, RedisError> =
        connection_client.sadd("key_6", vec!["member_1", "member_2"]);

    let received_1: Result<isize, RedisError> = redis::cmd("incrby")
        .arg("key_1")
        .arg("10")
        .query(&mut connection_client);
    let received_2: Result<isize, RedisError> = redis::cmd("incrby")
        .arg("key_2")
        .arg("10")
        .query(&mut connection_client);
    let received_3: Result<isize, RedisError> = redis::cmd("incrby")
        .arg("key_3")
        .arg("10")
        .query(&mut connection_client);
    let received_4: Result<isize, RedisError> = redis::cmd("incrby")
        .arg("key_4")
        .arg("10")
        .query(&mut connection_client);
    let received_5: Result<isize, RedisError> = redis::cmd("incrby")
        .arg("key_5")
        .arg("10")
        .query(&mut connection_client);
    let received_6: Result<isize, RedisError> = redis::cmd("incrby")
        .arg("key_6")
        .arg("10")
        .query(&mut connection_client);
    assert_eq!(received_1, Ok(20));
    assert_eq!(received_2, Ok(10));
    assert_eq!(received_3, Ok(0));
    assert!(received_4.is_err());
    assert!(received_5.is_err());
    assert!(received_6.is_err());

    let received_1_get: Result<String, RedisError> = connection_client.get("key_1");
    let received_2_get: Result<String, RedisError> = connection_client.get("key_2");
    let received_3_get: Result<String, RedisError> = connection_client.get("key_3");
    assert_eq!(received_1_get, Ok("20".to_string()));
    assert_eq!(received_2_get, Ok("10".to_string()));
    assert_eq!(received_3_get, Ok("0".to_string()));

    server.shutdown()?;
    Ok(())
}

#[ignore = "Integration Test"]
#[test]
fn int_test_08_decrby() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "10");
    let _: Result<String, RedisError> = connection_client.set("key_2", "0");
    let _: Result<String, RedisError> = connection_client.set("key_3", "-10");
    let _: Result<String, RedisError> = connection_client.set("key_4", "value");
    let _: Result<String, RedisError> = connection_client.lpush("key_5", vec!["item_1", "item_2"]);
    let _: Result<String, RedisError> =
        connection_client.sadd("key_6", vec!["member_1", "member_2"]);

    let received_1: Result<isize, RedisError> = redis::cmd("decrby")
        .arg("key_1")
        .arg("10")
        .query(&mut connection_client);
    let received_2: Result<isize, RedisError> = redis::cmd("decrby")
        .arg("key_2")
        .arg("10")
        .query(&mut connection_client);
    let received_3: Result<isize, RedisError> = redis::cmd("decrby")
        .arg("key_3")
        .arg("10")
        .query(&mut connection_client);
    let received_4: Result<isize, RedisError> = redis::cmd("decrby")
        .arg("key_4")
        .arg("10")
        .query(&mut connection_client);
    let received_5: Result<isize, RedisError> = redis::cmd("decrby")
        .arg("key_5")
        .arg("10")
        .query(&mut connection_client);
    let received_6: Result<isize, RedisError> = redis::cmd("decrby")
        .arg("key_6")
        .arg("10")
        .query(&mut connection_client);
    assert_eq!(received_1, Ok(0));
    assert_eq!(received_2, Ok(-10));
    assert_eq!(received_3, Ok(-20));
    assert!(received_4.is_err());
    assert!(received_5.is_err());
    assert!(received_6.is_err());

    let received_1_get: Result<String, RedisError> = connection_client.get("key_1");
    let received_2_get: Result<String, RedisError> = connection_client.get("key_2");
    let received_3_get: Result<String, RedisError> = connection_client.get("key_3");
    assert_eq!(received_1_get, Ok("0".to_string()));
    assert_eq!(received_2_get, Ok("-10".to_string()));
    assert_eq!(received_3_get, Ok("-20".to_string()));

    server.shutdown()?;
    Ok(())
}

#[ignore = "Integration Test"]
#[test]
fn int_test_09_getset() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value_1_old");
    let _: Result<String, RedisError> = connection_client.lpush("key_2", vec!["item_1", "item_2"]);
    let _: Result<String, RedisError> =
        connection_client.sadd("key_3", vec!["member_1", "member_2"]);

    let received_1: Result<String, RedisError> = connection_client.getset("key_1", "value_1_new");
    let received_2: Result<String, RedisError> = connection_client.getset("key_2", "item_3");
    let received_3: Result<String, RedisError> = connection_client.getset("key_3", "member_3");
    assert_eq!(received_1, Ok("value_1_old".to_string()));
    assert!(received_2.is_err());
    assert!(received_3.is_err());

    let received_1_get: Result<String, RedisError> = connection_client.get("key_1");
    assert_eq!(received_1_get, Ok("value_1_new".to_string()));

    server.shutdown()?;
    Ok(())
}

#[ignore = "Integration Test"]
#[test]
fn int_test_10_getdel() -> Result<(), ErrorStruct> {
    let mut server = setup::ServerTest::start()?;
    let mut connection_client = server.get_connection_client()?;
    let _: Result<String, RedisError> = connection_client.set("key_1", "value_1");
    let _: Result<String, RedisError> = connection_client.lpush("key_2", vec!["item_1", "item_2"]);
    let _: Result<String, RedisError> =
        connection_client.sadd("key_3", vec!["member_1", "member_2"]);

    let received_1: Result<String, RedisError> = redis::cmd("getdel")
        .arg("key_1")
        .query(&mut connection_client);

    let received_2: Result<String, RedisError> = redis::cmd("getdel")
        .arg("key_2")
        .query(&mut connection_client);

    let received_3: Result<String, RedisError> = redis::cmd("getdel")
        .arg("key_3")
        .query(&mut connection_client);

    let received_4: Result<String, RedisError> = redis::cmd("getdel")
        .arg("key_4")
        .query(&mut connection_client);

    assert_eq!(received_1, Ok("value_1".to_string()));
    assert!(received_2.is_err());
    assert!(received_3.is_err());
    assert!(received_4.is_err());

    let received_1_get: Result<String, RedisError> = connection_client.get("key_1");
    let received_2_get: Result<String, RedisError> = connection_client.get("key_2");
    let received_3_get: Result<String, RedisError> = connection_client.get("key_3");
    let received_4_get: Result<String, RedisError> = connection_client.get("key_4");
    assert!(received_1_get.is_err());
    assert!(received_2_get.is_err());
    assert!(received_3_get.is_err());
    assert!(received_4_get.is_err());

    server.shutdown()?;
    Ok(())
}
