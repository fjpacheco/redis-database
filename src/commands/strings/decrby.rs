use super::execute_value_modification;
use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::Runnable, database::Database, messages::redis_messages,
    native_types::error::ErrorStruct,
};
use std::sync::{Arc, Mutex};

pub struct Decrby;

/// Decrements the number stored at key by decrement. If the key does not exist, it is set
/// to 0 before performing the operation. An error is returned if the key contains a value
/// of the wrong type or contains a string that can not be represented as integer.
///
/// Operation is limited to 64 bit signed integers.

impl Runnable<Arc<Mutex<Database>>> for Decrby {
    fn run(
        &self,
        buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        execute_value_modification(&mut database, buffer, decr)
    }
}

fn decr(minuend: isize, subtrahend: isize) -> isize {
    minuend - subtrahend
}

#[cfg(test)]
pub mod test_decrby {
    use crate::commands::create_notifier;

    use crate::{database::TypeSaved, vec_strings};

    use super::*;

    #[test]
    fn test01_decrby_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> DECRBY mykey 3 ---> (integer) 7
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Decrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":7\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("7".to_string()))
        );
    }

    #[test]
    fn test02_decrby_existing_key_by_negative_integer() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> DECRBY mykey -3
        let buffer = vec_strings!["mykey", "-3"];
        let encoded = Decrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("13".to_string()))
        );
    }

    #[test]
    fn test03_decrby_existing_key_with_negative_integer_value() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey -10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("-10".to_string()));
        // redis> DECRBY mykey 3
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Decrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-13\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("-13".to_string()))
        );
    }

    #[test]
    fn test04_decrby_existing_key_with_negative_integer_value_by_negative_integer() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey -10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("-10".to_string()));
        // redis> DECRBY mykey -3
        let buffer = vec_strings!["mykey", "-3"];
        let encoded = Decrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-7\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("-7".to_string()))
        );
    }

    #[test]
    fn test05_decrby_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Decrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-3\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("-3".to_string()))
        );
    }

    #[test]
    fn test06_decrby_existing_key_with_non_decrementable_value() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey value
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("value".to_string()));
        // redis> DECRBY mykey 1
        let buffer = vec_strings!["mykey", "value"];
        let error = Decrby.run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }

    #[test]
    fn test07_decrby_existing_key_by_non_integer() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> DECRBY mykey a
        let buffer = vec_strings!["mykey", "a"];
        let error = Decrby.run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }
}
