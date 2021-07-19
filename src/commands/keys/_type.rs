use crate::{
    commands::{check_empty_2, check_not_empty, Runnable},
    database::{Database, TypeSaved},
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct Type;

impl Runnable<Database> for Type {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_not_empty(&buffer)?;
        let key = buffer.pop().unwrap();
        check_empty_2(&buffer)?;
        if let Some(typesaved) = database.get(&key) {
            match typesaved {
                TypeSaved::String(_) => Ok(RSimpleString::encode("string".to_string())),
                TypeSaved::Set(_) => Ok(RSimpleString::encode("set".to_string())),
                TypeSaved::List(_) => Ok(RSimpleString::encode("list".to_string())),
            }
        } else {
            Ok(RSimpleString::encode("none".to_string()))
        }
    }
}

#[cfg(test)]
mod test_type {

    use std::collections::VecDeque;

    use super::*;
    use crate::{
        commands::{create_notifier, sets::sadd::Sadd},
        database::TypeSaved,
        native_types::RSimpleString,
        vec_strings,
    };

    #[test]
    fn test01_type_of_string_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer = vec_strings!["key"];
        let result = Type.run(buffer, &mut database);
        assert_eq!(RSimpleString::encode("string".to_string()), result.unwrap());
    }

    #[test]
    fn test02_type_of_set_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        let buffer1 = vec_strings!["key", "member1", "member2"];
        let _result1 = Sadd.run(buffer1, &mut database);

        let buffer2 = vec_strings!["key"];
        let result2 = Type.run(buffer2, &mut database);
        assert_eq!(RSimpleString::encode("set".to_string()), result2.unwrap());
    }

    #[test]
    fn test03_type_of_list_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        database.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key"];
        let result = Type.run(buffer, &mut database);
        assert_eq!(RSimpleString::encode("list".to_string()), result.unwrap());
    }

    #[test]
    fn test01_type_of_non_existent_string_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);
        database.insert("key1".to_string(), TypeSaved::String("value".to_string()));
        let buffer = vec_strings!["key2"];
        let result = Type.run(buffer, &mut database);
        assert_eq!(RSimpleString::encode("none".to_string()), result.unwrap());
    }
}
