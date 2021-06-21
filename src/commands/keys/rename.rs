use crate::{
    commands::{check_empty_2, check_not_empty, Runnable},
    database::Database,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct Rename;

/// Renames key to newkey. It returns an error when key does not exist.
/// If newkey already exists it is overwritten, when this happens RENAME
/// executes an implicit DEL operation, so if the deleted key contains a
/// very big value it may cause high latency even if RENAME itself is
/// usually a constant-time operation.

impl Runnable<Database> for Rename {
    fn run(
        &self,
        mut _buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_not_empty(&_buffer_vec)?;
        let new_key = _buffer_vec.pop().unwrap();
        check_not_empty(&_buffer_vec)?;
        let old_key = _buffer_vec.pop().unwrap();
        check_empty_2(&_buffer_vec)?;
        if let Some(string_list) = database.get(old_key) {
            let value = string_list.clone();
            database.remove(old_key);
            database.insert(new_key.to_string(), value);
            Ok(RSimpleString::encode("OK".to_string()))
        } else {
            Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("no such key"),
            ))
        }
    }
}

#[cfg(test)]
mod test_rename {

    use super::*;
    use crate::{commands::strings::get::Get, database::TypeSaved, native_types::RBulkString};

    #[test]
    fn test01_rename_existing_key_with_new_key() {
        let mut database = Database::new();
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock_1 = vec!["key", "new_key"];
        let result1 = Rename.run(buffer_vec_mock_1, &mut database);
        assert_eq!(result1.unwrap(), "+OK\r\n".to_string());
        let buffer_vec_mock_2 = vec!["new_key"];
        let result2 = Get.run(buffer_vec_mock_2, &mut database);
        assert_eq!(RBulkString::encode("value".to_string()), result2.unwrap());
    }

    #[test]
    fn test02_rename_non_existing_key() {
        let mut database = Database::new();
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock = vec!["random_key", "new_key"];
        let error = Rename.run(buffer_vec_mock, &mut database);
        assert_eq!(error.unwrap_err().print_it(), "ERR no such key".to_string());
    }

    #[test]
    fn test03_rename_existing_key_with_existing_key() {
        let mut database = Database::new();
        database.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock_1 = vec!["key", "key"];
        let result1 = Rename.run(buffer_vec_mock_1, &mut database);
        assert_eq!(result1.unwrap(), "+OK\r\n".to_string());
        let buffer_vec_mock_2 = vec!["key"];
        let result2 = Get.run(buffer_vec_mock_2, &mut database);
        assert_eq!(RBulkString::encode("value".to_string()), result2.unwrap());
    }
}
