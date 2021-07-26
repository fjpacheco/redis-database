use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
use crate::{
    commands::lists::{check_empty, check_not_empty},
    native_types::{error::ErrorStruct, redis_type::RedisType, simple_string::RSimpleString},
};
use crate::{
    commands::{get_as_integer, Runnable},
    database::{Database, TypeSaved},
    native_types::RInteger,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct Lrem;

impl Runnable<Arc<Mutex<Database>>> for Lrem {
    /// Removes the first count occurrences of elements equal to element from the list
    /// stored at key. The count argument influences the operation in the following ways:
    /// * count > 0: Remove elements equal to element moving from head to tail.
    /// * count < 0: Remove elements equal to element moving from tail to head.
    /// * count = 0: Remove all elements equal to element.
    /// For example, LREM list -2 "hello" will remove the last two occurrences of "hello"
    /// in the list stored at list.
    /// Note that non-existing keys are treated like empty lists, so when key does not
    /// exist, the command will always return 0.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: the number of removed elements.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a list.
    /// * Buffer [Vec]<[String]> is received empty, or received with an amount of elements different than 3.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        mut buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        check_empty(&buffer, "lrem")?;
        let key = buffer.remove(0);
        check_empty(&buffer, "lrem")?;
        let value = buffer.pop().unwrap();
        check_empty(&buffer, "lrem")?;
        let count = get_as_integer(&buffer.pop().unwrap()).unwrap();
        check_not_empty(&buffer)?;
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

// Removes the received value according to the count sign.
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

// Receives a negative isize count and iterates the VecDeque from tail to head
// removing as many repetitions of the value as count indicates.
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

// Receives a positive (or zero) isize count and iterates the VecDeque from head
// to tail removing as many repetitions of the value as count indicates.
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
    use crate::commands::create_notifier;

    use crate::{
        commands::lists::lrange::Lrange,
        database::{Database, TypeSaved},
        vec_strings,
    };
    use std::collections::VecDeque;

    use super::*;
    #[test]
    fn test_01_lrem_negative_count() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("hello".to_string());
        new_list.push_back("hello".to_string());
        new_list.push_back("foo".to_string());
        new_list.push_back("hello".to_string());

        let key = "key".to_string();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        // redis> LREM mylist -2 "hello"
        let buffer1 = vec_strings!["key", "-2", "hello"];
        let encoded1 = Lrem.run(buffer1, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded1.unwrap(), ":2\r\n".to_string()); // (integer) 2

        // redis> LRANGE mylist 0 -1
        let buffer2 = vec_strings!["key", "0", "-1"];
        let encoded2 = Lrange.run(buffer2, &mut data);
        assert_eq!(
            encoded2.unwrap().to_string(),
            "*2\r\n$10\r\n1) \"hello\"\r\n$8\r\n2) \"foo\"\r\n".to_string(),
        );
    }

    #[test]
    fn test_02_lrem_positive_count() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("hello".to_string());
        new_list.push_back("hello".to_string());
        new_list.push_back("foo".to_string());
        new_list.push_back("hello".to_string());

        let key = "key".to_string();
        // let key_cpy = key.clone();
        data.lock().unwrap().insert(key, TypeSaved::List(new_list));

        // redis> LREM mylist -2 "hello"
        let buffer1 = vec_strings!["key", "2", "hello"];
        let encoded1 = Lrem.run(buffer1, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded1.unwrap(), ":2\r\n".to_string()); // (integer) 2

        // redis> LRANGE mylist 0 -1
        let buffer2 = vec_strings!["key", "0", "-1"];
        let encoded2 = Lrange.run(buffer2, &mut data);
        assert_eq!(
            encoded2.unwrap().to_string(),
            "*2\r\n$8\r\n1) \"foo\"\r\n$10\r\n2) \"hello\"\r\n".to_string(),
        );
    }

    #[test]
    fn test_02_lrem_count_equals_zero() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("hello".to_string());
        new_list.push_back("hello".to_string());
        new_list.push_back("foo".to_string());
        new_list.push_back("hello".to_string());

        let key = "key".to_string();
        // let key_cpy = key.clone();
        {
            data.lock().unwrap().insert(key, TypeSaved::List(new_list));
        }
        // redis> LREM mylist -2 "hello"
        let buffer1 = vec_strings!["key", "0", "hello"];
        let encoded1 = Lrem.run(buffer1, &mut data);

        // Check return value is simple string OK
        assert_eq!(encoded1.unwrap(), ":3\r\n".to_string()); // (integer) 2

        // redis> LRANGE mylist 0 -1
        let buffer2 = vec_strings!["key", "0", "-1"];
        let encoded2 = Lrange.run(buffer2, &mut data);
        assert_eq!(
            encoded2.unwrap().to_string(),
            "*1\r\n$8\r\n1) \"foo\"\r\n".to_string(),
        );
    }
}
