use crate::commands::keys::{no_more_values, pop_value};
use crate::commands::Runnable;
use crate::database::Database;
use crate::messages::redis_messages::{self, wrong_regex_pattern};
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::ErrorStruct;
use crate::native_types::RArray;
use crate::native_types::RedisType;
use std::sync::{Arc, Mutex};

pub struct Keys;

impl Runnable<Arc<Mutex<Database>>> for Keys {
    /// Returns all keys matching pattern.
    /// Warning: consider KEYS as a command that should only be used in production
    /// This command is intended for debugging and special operations, such as
    /// changing your keyspace layout. Don't use KEYS in your regular application
    /// code.
    ///
    /// # Return value
    /// * [String] _encoded_ in [RArray]: list of keys matching pattern.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty, or received with more than 1 element.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        mut buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        let regex = pop_value(&mut buffer, "keys")?;
        no_more_values(&buffer, "keys")?;

        match database.match_pattern(&regex) {
            Ok(vec) => Ok(RArray::encode(vec)),
            Err(_) => Err(ErrorStruct::from(wrong_regex_pattern(&regex))),
        }
    }
}

#[cfg(test)]
mod test_keys {

    use super::*;
    use crate::{commands::create_notifier, database::TypeSaved};

    fn default_database() -> Database {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut db = Database::new(notifier);

        db.insert(String::from("Camo"), TypeSaved::String(String::from("a")));
        db.insert(String::from("Cemo"), TypeSaved::String(String::from("a")));
        db.insert(String::from("Cimo"), TypeSaved::String(String::from("a")));
        db.insert(String::from("Como"), TypeSaved::String(String::from("a")));
        db.insert(String::from("David"), TypeSaved::String(String::from("a")));
        db.insert(String::from("Hello"), TypeSaved::String(String::from("a")));
        db.insert(
            String::from("Hassallo"),
            TypeSaved::String(String::from("a")),
        );
        db.insert(
            String::from("dsaHello"),
            TypeSaved::String(String::from("a")),
        );
        db.insert(String::from("Hollo"), TypeSaved::String(String::from("a")));
        db.insert(
            String::from("Hiaillo"),
            TypeSaved::String(String::from("a")),
        );

        db
    }

    #[test]

    fn test_01_match_one_character() {
        let db = default_database();

        let mut matched = db.match_pattern("C?mo").unwrap();
        matched.sort();

        assert_eq!(&matched[0], "Camo");
        assert_eq!(&matched[1], "Cemo");
        assert_eq!(&matched[2], "Cimo");
        assert_eq!(&matched[3], "Como");
        assert_eq!(matched.get(4), None);
    }

    #[test]

    fn test_02_match_a_range_of_character() {
        let db = default_database();

        let mut matched = db.match_pattern("H*").unwrap();
        matched.sort();

        assert_eq!(&matched[0], "Hassallo");
        assert_eq!(&matched[1], "Hello");
        assert_eq!(&matched[2], "Hiaillo");
        assert_eq!(&matched[3], "Hollo");
        assert_eq!(matched.get(4), None);
    }
}
