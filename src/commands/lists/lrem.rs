use std::collections::VecDeque;

use crate::native_types::redis_type::RedisType;
use crate::native_types::{error::ErrorStruct, simple_string::RSimpleString};
use crate::{
    commands::get_as_integer,
    database::{Database, TypeSaved},
    native_types::RInteger,
};

pub struct Lrem;

// Removes the first count occurrences of elements equal to element from the list
// stored at key. The count argument influences the operation in the following ways:
// count > 0: Remove elements equal to element moving from head to tail.
// count < 0: Remove elements equal to element moving from tail to head.
// count = 0: Remove all elements equal to element.

impl Lrem {
    pub fn run(mut buffer: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        let key = String::from(buffer.remove(0));
        let value = String::from(buffer.pop().unwrap());
        let count = get_as_integer(buffer.pop().unwrap()).unwrap();
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::List(values_list) => remove_value(count, value, values_list),
                _ => Err(ErrorStruct::new(
                    String::from("WRONGTYPE"),
                    String::from("Operation against a key holding the wrong kind of value"),
                )),
            }
        } else {
            // non-existing keys are treated like empty lists (return 0)
            // (integer) 0
            Ok(RSimpleString::encode("(integer) 0".to_string())) // Check
        }
    }
}

fn remove_value(
    count: isize,
    value: String,
    values_list: &mut VecDeque<String>,
) -> Result<String, ErrorStruct> {
    match count {
        count if count < 0 => remove_value_negative_count(count, value, values_list),
        _ => remove_value_default(count, value, values_list),
    }
}
#[allow(dead_code)]
pub fn remove_value_negative_count(
    count: isize,
    value: String,
    values_list: &mut VecDeque<String>,
) -> Result<String, ErrorStruct> {
    // if count < 0, iterate from tail to head
    let mut i = 0;
    let mut j = values_list.len() - 1;
    let mut values_list_clone = values_list.clone();
    for element in values_list_clone.iter_mut().rev() {
        if *element == value {
            values_list.remove(j);
            i += 1;
        }
        if i == count.abs() {
            break;
        }
        j -= 1;
    }
    Ok(RInteger::encode(i as isize))
}

#[allow(dead_code)]
pub fn remove_value_default(
    count: isize,
    value: String,
    values_list: &mut VecDeque<String>,
) -> Result<String, ErrorStruct> {
    // if count >= 0, iterate from head to tail
    let mut i = 0;
    let mut j = 0;
    let mut values_list_clone = values_list.clone();
    for element in values_list_clone.iter_mut() {
        if *element == value {
            values_list.remove(j);
            i += 1;
        } else {
            j += 1;
        }
        if count != 0 && i == count.abs() {
            break;
        }
    }
    Ok(RInteger::encode(i as isize))
}

#[cfg(test)]
pub mod test_lset {

    use crate::{
        commands::lists::lrange::Lrange,
        database::{Database, TypeSaved},
    };
    use std::collections::VecDeque;

    use super::Lrem;
    #[test]
    fn test01_lrem_negative_count() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("hello".to_string());
        new_list.push_back("hello".to_string());
        new_list.push_back("foo".to_string());
        new_list.push_back("hello".to_string());

        let key = "key".to_string();
        data.insert(key, TypeSaved::List(new_list));

        // redis> LREM mylist -2 "hello"
        let buffer1 = vec!["key", "-2", "hello"];
        let encoded1 = Lrem::run(buffer1, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded1.unwrap(), ":2\r\n".to_string()); // (integer) 2

        // redis> LRANGE mylist 0 -1
        let buffer2 = vec!["key", "0", "-1"];
        let encoded2 = Lrange::run(buffer2, &mut data);
        assert_eq!(
            encoded2.unwrap().to_string(),
            "*2\r\n$10\r\n1) \"hello\"\r\n$8\r\n2) \"foo\"\r\n".to_string(),
        );
    }

    #[test]
    fn test02_lrem_positive_count() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("hello".to_string());
        new_list.push_back("hello".to_string());
        new_list.push_back("foo".to_string());
        new_list.push_back("hello".to_string());

        let key = "key".to_string();
        // let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        // redis> LREM mylist -2 "hello"
        let buffer1 = vec!["key", "2", "hello"];
        let encoded1 = Lrem::run(buffer1, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded1.unwrap(), ":2\r\n".to_string()); // (integer) 2

        // redis> LRANGE mylist 0 -1
        let buffer2 = vec!["key", "0", "-1"];
        let encoded2 = Lrange::run(buffer2, &mut data);
        assert_eq!(
            encoded2.unwrap().to_string(),
            "*2\r\n$8\r\n1) \"foo\"\r\n$10\r\n2) \"hello\"\r\n".to_string(),
        );
    }

    #[test]
    fn test02_lrem_count_equals_zero() {
        let mut data = Database::new();

        let mut new_list = VecDeque::new();
        new_list.push_back("hello".to_string());
        new_list.push_back("hello".to_string());
        new_list.push_back("foo".to_string());
        new_list.push_back("hello".to_string());

        let key = "key".to_string();
        // let key_cpy = key.clone();
        data.insert(key, TypeSaved::List(new_list));

        // redis> LREM mylist -2 "hello"
        let buffer1 = vec!["key", "0", "hello"];
        let encoded1 = Lrem::run(buffer1, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded1.unwrap(), ":3\r\n".to_string()); // (integer) 2

        // redis> LRANGE mylist 0 -1
        let buffer2 = vec!["key", "0", "-1"];
        let encoded2 = Lrange::run(buffer2, &mut data);
        assert_eq!(
            encoded2.unwrap().to_string(),
            "*1\r\n$8\r\n1) \"foo\"\r\n".to_string(),
        );
    }
}
