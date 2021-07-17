use super::{no_more_values, pop_value};
use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RInteger, RedisType},
    Database,
};
pub struct Persist;

impl Runnable<Database> for Persist {
    fn run(&self, mut buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        let key = pop_value(&mut buffer, "Persist")?;
        no_more_values(&buffer, "Persist")?;

        if database.persist(&key).is_some() {
            Ok(RInteger::encode(1))
        } else {
            Ok(RInteger::encode(0))
        }
    }
}
