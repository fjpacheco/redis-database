use std::collections::HashMap;

use crate::{database::RedisTypes, native_types::NativeTypes};

#[derive(Debug)]
pub struct RedisString {
    value: String,
}

impl RedisString {
    #[allow(dead_code)]
    fn as_native_type(&self) -> NativeTypes {
        NativeTypes::new_bulk_string(self.value.as_str())
    }

    pub fn run(
        buffer: String,
        database: &mut HashMap<String, RedisTypes>,
    ) -> Result<NativeTypes, NativeTypes> {
        let mut buffer_split = buffer.split_whitespace();
        let command = buffer_split.next().unwrap_or("");
        match command {
            "set" => {
                let key = buffer_split.next().unwrap_or("");
                let value = buffer_split.next().unwrap_or("");
                let redis_string = Ok((
                    RedisString {
                        value: value.to_string(),
                    },
                    NativeTypes::new_simple_string("OK"),
                ));
                match redis_string {
                    Ok(value) => {
                        database.insert(key.to_string(), RedisTypes::String(value.0));
                        Ok(value.1)
                    }
                    Err(err) => Err(err),
                }
            }
            _ => Err(NativeTypes::new_error("ERR Command not found ")),
        }
    }
}
/*
#[cfg(test)]
mod test_decode {
    use super::*;

    #[test]
    fn test01_set_redis_string_return_simple_string_with_ok() {
        // Asumiendo que, Servidor recibió bytes, parrseo, agarró argv[1], argv[2], argv[3] y se almacenó esas 3 cositas en 3 varibales command, key y value.
        //let command =  String::from("set");
        //let     key =  String::from("diego");
        let value = String::from("10");

        let redis_string = RedisString::set(value);
        let status_set = match redis_string {
            Ok(item_ok) => item_ok.1,
            Err(item_err) => item_err,
        };

        assert_eq!("+OK\r\n".to_string(), status_set.encode().unwrap());
    }

    #[test]
    fn test02_get_redis_string_return_bulk_string_with_value() {
        // Asumiendo que, Servidor recibió bytes, parrseo, agarró argv[1], argv[2], argv[3] y se almacenó esas 3 cositas en 3 varibales command, key y value.
        //let command =  String::from("set");
        //let     key =  String::from("diego");
        let value = String::from("10");

        let redis_string = RedisString::set(value).unwrap().0;
        let variable_get = redis_string.get();

        assert_eq!("$2\r\n10\r\n".to_string(), variable_get.encode().unwrap());
    }
}
*/
