use crate::commands::{check_empty, Runnable};
use crate::database::{Database, TypeSaved};
use crate::native_types::error::ErrorStruct;
use crate::native_types::integer::RInteger;
use crate::native_types::redis_type::RedisType;

use super::{no_more_values, pop_value};

pub struct Append;
impl Runnable<Database> for Append {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "append")?;

        let new_value = pop_value(&mut buffer)?;
        let key = pop_value(&mut buffer)?;
        no_more_values(&buffer, "append")?;
        let size: usize;

        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::String(old_value) => {
                    old_value.push_str(&new_value);
                    size = old_value.len();
                    Ok(RInteger::encode(size as isize))
                }

                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from strings"),
                )),
            }
        } else {
            size = new_value.len();
            database.insert(key, TypeSaved::String(new_value));
            Ok(RInteger::encode(size as isize))
        }
    }
}

#[cfg(test)]
pub mod test_append {

    use super::*;
    use crate::{
        database::{Database, TypeSaved},
        vec_strings,
    };

    #[test]
    fn test01_append_to_an_existing_key() {
        let mut data = Database::new();

        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key", "Appended"];
        let encoded = Append.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.get("key"),
            Some(&TypeSaved::String("valueAppended".to_string()))
        );
    }

    #[test]
    fn test02_append_to_a_non_existing_key() {
        let mut data = Database::new();
        let buffer = vec_strings!["key", "newValue"];
        let encoded = Append.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":8\r\n".to_string());
        assert_eq!(
            data.get("key"),
            Some(&TypeSaved::String("newValue".to_string()))
        );
    }

    #[test]
    fn test03_wrong_number_of_arguments() {
        let mut data = Database::new();

        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key"];
        let encoded = Append.run(buffer, &mut data);
        match encoded {
            Ok(_value) => {}
            Err(error) => assert_eq!(
                error.print_it(),
                "ERR wrong number of arguments for 'append' command"
            ),
        }
    }
}
