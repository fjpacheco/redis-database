use crate::{
    commands::Runnable,
    native_types::ErrorStruct,
    native_types::{RSimpleString, RedisType},
    Database,
};
pub struct FlushDB;

impl Runnable<Database> for FlushDB {
    fn run(&self, _buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        database.clear();
        Ok(RSimpleString::encode("OK".to_string()))
    }
}
