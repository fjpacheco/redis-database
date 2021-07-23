use super::{no_more_values, pop_value, replace_value};
use crate::database::{Database, TypeSaved};
use crate::native_types::bulk_string::RBulkString;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::redis_type::RedisType;
use crate::{commands::Runnable, messages::redis_messages};
use std::sync::{Arc, Mutex};

pub struct Getset;

impl Runnable<Arc<Mutex<Database>>> for Getset {
    fn run(
        &self,
        mut buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        let new_value = pop_value(&mut buffer)?;
        let key = pop_value(&mut buffer)?;
        no_more_values(&buffer, "getset")?;

        if let Some(typesaved) = database.get(&key) {
            match typesaved {
                TypeSaved::String(_old_value) => replace_value(&mut database, key, new_value),
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided does not match an string"),
                )),
            }
        } else {
            database.insert(key, TypeSaved::String(new_value));
            Ok(RBulkString::encode("(nil)".to_string()))
        }
    }
}

#[cfg(test)]
pub mod test_getset {
    use crate::commands::create_notifier;

    use super::*;
    use crate::{
        database::{Database, TypeSaved},
        vec_strings,
    };
    #[test]
    fn test_01_getset_of_an_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key", "other"];
        let encoded = Getset.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), "$5\r\nvalue\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("key"),
            Some(&TypeSaved::String("other".to_string()))
        );
    }

    #[test]
    fn test_02_getset_of_a_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["key", "newValue"];
        let encoded = Getset.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), "$-1\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("key"),
            Some(&TypeSaved::String("newValue".to_string()))
        );
    }

    #[test]
    fn test_03_wrong_number_of_arguments() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings![];
        let encoded = Getset.run(buffer, &mut data);
        match encoded {
            Ok(_value) => {}
            Err(error) => assert_eq!(
                error.print_it(),
                "ERR wrong number of arguments for 'append' command"
            ),
        }
    }
}
