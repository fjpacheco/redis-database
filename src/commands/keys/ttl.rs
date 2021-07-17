use super::{no_more_values, pop_value};
use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RInteger, RedisType},
    Database,
};
pub struct Ttl;

impl Runnable<Database> for Ttl {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        let key = pop_value(&mut buffer, "Ttl")?;
        no_more_values(&buffer, "Ttl")?;

        if database.contains_key(&key) {
            if let Some(ttl) = database.ttl(&key) {
                Ok(RInteger::encode(ttl as isize))
            } else {
                Ok(RInteger::encode(-1))
            }
        } else {
            Ok(RInteger::encode(-2))
        }
    }
}
