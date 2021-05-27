use super::database::{execute_value_modification, Database};
use crate::native_types::error::ErrorStruct;

pub struct Incrby;

/// Increments the number stored at key by increment. If the key does not exist, it is set
/// to 0 before performing the operation. An error is returned if the key contains a value
/// of the wrong type or contains a string that can not be represented as integer.
///
/// This operation is limited to 64 bit signed integers.

impl Incrby {
    pub fn run(buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        execute_value_modification(database, buffer_vec, incr)
    }
}

fn incr(addend1: isize, addend2: isize) -> isize {
    addend1 + addend2
}

#[cfg(test)]
pub mod test_decrby {

    use super::*;

    #[test]
    fn test01_incrby_existing_key() {
        let mut data = Database::new();
        {
            // redis> SET mykey "10" ---> "OK"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> INCRBY mykey 3 ---> (integer) 13
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Incrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"13".to_string())
        );
    }

    #[test]
    fn test02_incrby_existing_key_by_negative_integer() {
        let mut data = Database::new();
        {
            // redis> SET mykey "10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> INCRBY mykey -3
        let buffer: Vec<&str> = vec!["mykey", "-3"];
        let encoded = Incrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":7\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"7".to_string())
        );
    }

    #[test]
    fn test03_incrby_existing_key_with_negative_integer_value() {
        let mut data = Database::new();
        {
            // redis> SET mykey "-10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "-10".to_string());
        }
        // redis> INCRBY mykey 3
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Incrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-7\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"-7".to_string())
        );
    }

    #[test]
    fn test04_incrby_existing_key_with_negative_integer_value_by_negative_integer() {
        let mut data = Database::new();
        {
            // redis> SET mykey "-10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "-10".to_string());
        }
        // redis> INCRBY mykey -3
        let buffer: Vec<&str> = vec!["mykey", "-3"];
        let encoded = Incrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-13\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"-13".to_string())
        );
    }

    #[test]
    fn test05_decrby_non_existing_key() {
        let mut data = Database::new();
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Incrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":3\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"3".to_string())
        );
    }

    #[test]
    fn test06_incrby_existing_key_with_non_decrementable_value() {
        let mut data = Database::new();
        {
            // redis> SET mykey value
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "value".to_string());
        }
        // redis> DECRBY mykey 1
        let buffer: Vec<&str> = vec!["mykey", "value"];
        let error = Incrby::run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }

    #[test]
    fn test07_decrby_existing_key_by_non_integer() {
        let mut data = Database::new();
        {
            // redis> SET mykey 10
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> INCRBY mykey a
        let buffer: Vec<&str> = vec!["mykey", "a"];
        let error = Incrby::run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }
}
