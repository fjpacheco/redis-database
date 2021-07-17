use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RInteger, RedisType},
    Database,
};
pub struct Touch;

impl Runnable<Database> for Touch {
    fn run(&self, buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        Ok(RInteger::encode(
            buffer
                .iter()
                .map(|key| if database.contains_key(key) { 1 } else { 0 })
                .sum(),
        ))
    }
}

#[cfg(test)]
pub mod test_touch {

    use crate::database::TypeSaved;

    use super::*;

    #[test]
    fn test01_three_keys_touch_three() {
        let mut db = Database::new();

        db.insert("key1".to_string(), TypeSaved::String("a".to_string()));
        db.insert("key2".to_string(), TypeSaved::String("b".to_string()));
        db.insert("key3".to_string(), TypeSaved::String("c".to_string()));

        let sum = Touch.run(
            vec!["key1".to_string(), "key2".to_string(), "key3".to_string()],
            &mut db,
        );

        assert_eq!(&sum.unwrap(), ":3\r\n");
    }

    #[test]
    fn test02_three_keys_touch_two() {
        let mut db = Database::new();

        db.insert("key1".to_string(), TypeSaved::String("a".to_string()));
        db.insert("key2".to_string(), TypeSaved::String("b".to_string()));
        db.insert("key3".to_string(), TypeSaved::String("c".to_string()));

        let sum = Touch.run(vec!["key1".to_string(), "key3".to_string()], &mut db);

        assert_eq!(&sum.unwrap(), ":2\r\n");
    }

    #[test]
    fn test03_three_keys_touch_four() {
        let mut db = Database::new();

        db.insert("key1".to_string(), TypeSaved::String("a".to_string()));
        db.insert("key2".to_string(), TypeSaved::String("b".to_string()));
        db.insert("key3".to_string(), TypeSaved::String("c".to_string()));

        let sum = Touch.run(
            vec![
                "key1".to_string(),
                "key2".to_string(),
                "key3".to_string(),
                "key4".to_string(),
            ],
            &mut db,
        );

        assert_eq!(&sum.unwrap(), ":3\r\n");
    }

    #[test]
    fn test04_three_keys_touch_zero() {
        let mut db = Database::new();

        db.insert("key1".to_string(), TypeSaved::String("a".to_string()));
        db.insert("key2".to_string(), TypeSaved::String("b".to_string()));
        db.insert("key3".to_string(), TypeSaved::String("c".to_string()));

        let sum = Touch.run(vec![], &mut db);

        assert_eq!(&sum.unwrap(), ":0\r\n");
    }
}
