mod support;

#[cfg(test)]

mod testings_redis {
    extern crate redis;

    use core::time;
    use std::thread;

    use redis::RedisError;
    use redis_rust::native_types::ErrorStruct;

    use crate::support::TestContext;

    #[test]
    //#[ignore] // TODO: Recordar hablar con Pablo al respecto de los Sleeps en Tests.
    fn test01_set_return_ok() -> Result<(), ErrorStruct> {
        let mut ctx = TestContext::new()?;
        let mut conection_client = ctx.connection();

        let received: Result<String, RedisError> = redis::cmd("set")
            .arg("key")
            .arg("value")
            .query(&mut conection_client);

        println!("Received: {:?}", received);
        assert_eq!(received.unwrap(), "OK");
        ctx.server_off();
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);
        Ok(())
    }
}
