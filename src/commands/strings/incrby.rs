use super::execute_value_modification;
use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::Runnable, database::Database, messages::redis_messages,
    native_types::error::ErrorStruct,
};
use std::sync::{Arc, Mutex};
pub struct Incrby;
impl Runnable<Arc<Mutex<Database>>> for Incrby {
    /// Increments the number stored at **key** by increment. If the **key** does not exist, it is set
    /// to 0 before performing the operation.
    ///
    /// This operation is limited to 64 bit signed integers.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger)(crate::native_types::integer::RInteger): the value of **key** after the increment.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The key contains a value of the wrong type or contains a string that can not be represented as integer.
    /// * The buffer [Vec]<[String]> more than two elements is received or empty.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
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
        execute_value_modification(&mut database, buffer, incr)
    }
}

fn incr(addend1: isize, addend2: isize) -> isize {
    addend1 + addend2
}

#[cfg(test)]
pub mod test_incrby {
    use crate::commands::create_notifier;

    use crate::{
        database::{Database, TypeSaved},
        vec_strings,
    };

    use super::*;

    #[test]
    fn test_01_incrby_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> INCRBY mykey 3 ---> (integer) 13
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("13".to_string()))
        );
    }

    #[test]
    fn test_02_incrby_existing_key_by_negative_integer() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> INCRBY mykey -3
        let buffer = vec_strings!["mykey", "-3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":7\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("7".to_string()))
        );
    }

    #[test]
    fn test_03_incrby_existing_key_with_negative_integer_value() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey -10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("-10".to_string()));
        // redis> INCRBY mykey 3
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-7\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("-7".to_string()))
        );
    }

    #[test]
    fn test_04_incrby_existing_key_with_negative_integer_value_by_negative_integer() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey -10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("-10".to_string()));
        // redis> INCRBY mykey -3
        let buffer = vec_strings!["mykey", "-3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-13\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("-13".to_string()))
        );
    }

    #[test]
    fn test_05_incrby_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":3\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("mykey"),
            Some(&TypeSaved::String("3".to_string()))
        );
    }

    #[test]
    fn test_06_incrby_existing_key_with_non_decrementable_value() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey value
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("value".to_string()));
        // redis> INCRBY mykey 1
        let buffer = vec_strings!["mykey", "value"];
        let error = Incrby.run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }

    #[test]
    fn test_07_decrby_existing_key_by_non_integer() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> INCRBY mykey a
        let buffer = vec_strings!["mykey", "a"];
        let error = Incrby.run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }
}
