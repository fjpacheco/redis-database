use crate::{
    commands::{check_empty_2, check_not_empty, Runnable},
    database::Database,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Exists;

impl Runnable<Database> for Exists {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_not_empty(&buffer)?;
        let key = buffer.pop().unwrap();
        check_empty_2(&buffer)?;
        if database.contains_key(&key) {
            Ok(RInteger::encode(1))
        } else {
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
mod test_exists {
    use crate::commands::create_notifier;

    use super::*;
    use crate::database::TypeSaved;

    #[test]
    fn test01_exists_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock = vec!["key".to_string()];
        let result_received = Exists.run(buffer_mock, &mut database);
        assert_eq!(RInteger::encode(1), result_received.unwrap());
    }

    #[test]
    fn test02_exists_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock = vec!["key1".to_string()];
        let result_received = Exists.run(buffer_mock, &mut database);
        assert_eq!(RInteger::encode(0), result_received.unwrap());
    }
}
