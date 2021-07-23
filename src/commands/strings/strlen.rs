use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};

use super::no_more_values;
use crate::commands::Runnable;
use crate::database::Database;
use crate::database::TypeSaved;
use crate::err_wrongtype;
use crate::messages::redis_messages;
use crate::native_types::{error::ErrorStruct, integer::RInteger, redis_type::RedisType};
pub struct Strlen;

impl Runnable<Arc<Mutex<Database>>> for Strlen {
    /// Returns the length of the string value stored at **key**.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger]:  the length of the string at key, or 0 when key does not exist.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Key holds a non-string value.
    /// * The buffer [Vec]<[String]> more than one element is received or empty.
    /// * [Database]  received in <[Arc]<[Mutex]>> is poisoned.
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
        let key = buffer.pop().unwrap();
        no_more_values(&buffer, "strlen")?;
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::String(old_value) => Ok(RInteger::encode(old_value.len() as isize)),
                _ => err_wrongtype!(),
            }
        } else {
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
pub mod test_strlen {

    use crate::commands::create_notifier;
    use crate::vec_strings;

    use super::*;

    #[test]
    fn test_01_strlen_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey somevalue ---> "OK"
        data.lock().unwrap().insert(
            "mykey".to_string(),
            TypeSaved::String("somevalue".to_string()),
        );
        // redis> STRLEN mykey ---> (integer) 9
        let buffer = vec_strings!["mykey"];
        let encoded = Strlen.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":9\r\n".to_string());
    }

    #[test]
    fn test_02_srlen_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> STRLEN nonexisting ---> (integer) 0
        let buffer = vec_strings!["mykey"];
        let encoded = Strlen.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":0\r\n".to_string());
    }
}
