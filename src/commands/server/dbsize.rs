use crate::{
    commands::Runnable,
    native_types::RedisType,
    native_types::{ErrorStruct, RInteger},
    Database,
};
pub struct Dbsize;

impl Runnable<Database> for Dbsize {
    fn run(&self, _buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        Ok(RInteger::encode(database.size() as isize))
    }
}
