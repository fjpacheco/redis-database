use crate::{
    commands::{check_empty_2, check_not_empty, Runnable},
    database::{Database, TypeSaved},
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct Type;

impl Runnable<Database> for Type {
    fn run(
        &self,
        mut _buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_not_empty(&_buffer_vec)?;
        let key = _buffer_vec.pop().unwrap();
        check_empty_2(&_buffer_vec)?;
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
    use crate::{commands::sets::sadd::Sadd, database::TypeSaved, native_types::RSimpleString};

    #[test]
    fn test01_type_of_string_key() {
        let mut database = Database::new();
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer = vec!["key"];
        let result = Type.run(buffer, &mut database);
        assert_eq!(RSimpleString::encode("string".to_string()), result.unwrap());
    }

    #[test]
    fn test02_type_of_set_key() {
        let mut database = Database::new();
        let buffer1 = vec!["key", "member1", "member2"];
        let _result1 = Sadd.run(buffer1, &mut database);

        let buffer2 = vec!["key"];
        let result2 = Type.run(buffer2, &mut database);
        assert_eq!(RSimpleString::encode("set".to_string()), result2.unwrap());
    }

    #[test]
    fn test03_type_of_list_key() {
        let mut database = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        database.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key"];
        let result = Type.run(buffer, &mut database);
        assert_eq!(RSimpleString::encode("list".to_string()), result.unwrap());
    }

    #[test]
    fn test01_type_of_non_existent_string_key() {
        let mut database = Database::new();
        database.insert("key1".to_string(), TypeSaved::String("value".to_string()));
        let buffer = vec!["key2"];
        let result = Type.run(buffer, &mut database);
        assert_eq!(RSimpleString::encode("none".to_string()), result.unwrap());
    }
}
