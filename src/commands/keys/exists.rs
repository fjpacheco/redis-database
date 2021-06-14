use crate::{
    commands::{check_empty_2, check_not_empty, Runnable},
    database::Database,
    native_types::{ErrorStruct, RInteger, RedisType},
};

pub struct Exists;

impl Runnable for Exists {
    fn run(
        &self,
        mut _buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_not_empty(&_buffer_vec)?;
        let key = _buffer_vec.pop().unwrap();
        check_empty_2(&_buffer_vec)?;
        if database.contains_key(key).is_some() {
            Ok(RInteger::encode(1))
        } else {
            Ok(RInteger::encode(0))
        }
    }
}

#[cfg(test)]
mod test_exists {

    use super::*;
    use crate::database::TypeSaved;

    #[test]
    fn test01_exists_existing_key() {
        let mut database = Database::new();
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock = vec!["key"];
        let result_received = Exists.run(buffer_vec_mock, &mut database);
        assert_eq!(RInteger::encode(1), result_received.unwrap());
    }

    #[test]
    fn test02_exists_non_existing_key() {
        let mut database = Database::new();
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock = vec!["key1"];
        let result_received = Exists.run(buffer_vec_mock, &mut database);
        assert_eq!(RInteger::encode(0), result_received.unwrap());
    }

}
