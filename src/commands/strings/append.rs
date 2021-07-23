use super::{no_more_values, pop_value};
use crate::commands::{check_empty, Runnable};
use crate::database::{Database, TypeSaved};
use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::integer::RInteger;
use crate::native_types::redis_type::RedisType;
use std::sync::{Arc, Mutex};

pub struct Append;
impl Runnable<Arc<Mutex<Database>>> for Append {
    /// If **key** already exists and is a string, this command appends the value at the end of the string.
    /// If key does not exist it is created and set as an empty string, so APPEND will be similar to SET in this special case.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger]: the length of the string after the append operation.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Key holds a non-string value.
    /// * The buffer [Vec]<[String]> more than two element is received or empty.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
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
        check_empty(&buffer, "append")?;

        let new_value = pop_value(&mut buffer)?;
        let key = pop_value(&mut buffer)?;
        no_more_values(&buffer, "append")?;
        let size: usize;

        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::String(old_value) => {
                    old_value.push_str(&new_value);
                    size = old_value.len();
                    Ok(RInteger::encode(size as isize))
                }

                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from strings"),
                )),
            }
        } else {
            size = new_value.len();
            database.insert(key, TypeSaved::String(new_value));
            Ok(RInteger::encode(size as isize))
        }
    }
}

#[cfg(test)]
pub mod test_append {
    use crate::commands::create_notifier;

    use super::*;
    use crate::{
        database::{Database, TypeSaved},
        vec_strings,
    };

    #[test]
    fn test_01_append_to_an_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key", "Appended"];
        let encoded = Append.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.lock().unwrap().get("key"),
            Some(&TypeSaved::String("valueAppended".to_string()))
        );
    }

    #[test]
    fn test_02_append_to_a_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["key", "newValue"];
        let encoded = Append.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":8\r\n".to_string());
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

        let buffer = vec_strings!["key"];
        let encoded = Append.run(buffer, &mut data);
        match encoded {
            Ok(_value) => {}
            Err(error) => assert_eq!(
                error.print_it(),
                "ERR wrong number of arguments for 'append' command"
            ),
        }
    }
}
