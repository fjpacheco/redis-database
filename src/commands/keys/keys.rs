use crate::commands::keys::{no_more_values, pop_value};
use crate::commands::Runnable;
use crate::messages::redis_messages::wrong_regex_pattern;
use crate::native_types::ErrorStruct;
use crate::native_types::RArray;
use crate::native_types::RedisType;
use crate::Database;

pub struct Keys;

impl Runnable<Database> for Keys {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
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
    use crate::database::TypeSaved;

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

    fn test01_match_one_character() {
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

    fn test02_match_a_range_of_character() {
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
