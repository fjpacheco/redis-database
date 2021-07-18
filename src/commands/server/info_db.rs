use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RArray, RedisType},
    Database,
};
pub struct InfoDB;

impl Runnable<Database> for InfoDB {
    fn run(&self, _buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        Ok(RArray::encode(database.info()?))
    }
}