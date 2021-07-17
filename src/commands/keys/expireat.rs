use super::{no_more_values, parse_integer, pop_value};
use crate::{
    commands::Runnable,
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RInteger, RedisType},
    Database,
};
pub struct ExpireAt;

impl Runnable<Database> for ExpireAt {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        let timeout = pop_value(&mut buffer, "Expire")?;
        if timeout.starts_with('-') {
            return Err(ErrorStruct::from(redis_messages::negative_number()));
        }
        let timeout = parse_integer(timeout)? as u64;
        let key = pop_value(&mut buffer, "Expire")?;
        no_more_values(&buffer, "Expire")?;

        check_errors(database.set_ttl_unix_timestamp(&key, timeout))
    }
}

fn check_errors(should_be_error: Result<(), ErrorStruct>) -> Result<String, ErrorStruct> {
    if let Err(error) = should_be_error {
        if let Some(prefix) = error.prefix() {
            match prefix {
                "TTL" => Err(error),
                _ => Ok(RInteger::encode(0)),
            }
        } else {
            Ok(RInteger::encode(0))
        }
    } else {
        Ok(RInteger::encode(1))
    }
}
