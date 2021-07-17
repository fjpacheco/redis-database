use crate::{
    commands::{check_empty_2, check_not_empty, Runnable},
    database::Database,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Del;

impl Runnable<Database> for Del {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_not_empty(&buffer)?;
        let key = buffer.pop().unwrap();
        check_empty_2(&buffer)?;
        if database.remove(&key).is_some() {
            Ok(RInteger::encode(1))
        } else {
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
mod test_del {
    use crate::commands::create_notifier;

    use super::*;
    use crate::{database::TypeSaved, vec_strings};

    #[test]
    fn test01_del_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_del = vec_strings!["key"];
        let result_received = Del.run(buffer_mock_del, &mut database);
        assert_eq!(RInteger::encode(1), result_received.unwrap());
    }

    #[test]
    fn test02_del_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_del = vec_strings!["key1"];
        let result_received = Del.run(buffer_mock_del, &mut database);
        assert_eq!(RInteger::encode(0), result_received.unwrap());
    }

    #[test]
    fn test01_del_key_just_deleted() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_mock_del_1 = vec_strings!["key"];
        let result1 = Del.run(buffer_mock_del_1, &mut database);
        assert_eq!(RInteger::encode(1), result1.unwrap());
        let buffer_mock_del_2 = vec_strings!["key"];
        let result2 = Del.run(buffer_mock_del_2, &mut database);
        assert_eq!(RInteger::encode(0), result2.unwrap());
    }
}
