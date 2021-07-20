
use crate::{
    commands::Runnable,
    database::Database,
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct Save;

impl Runnable<Database> for Save {
    fn run(&self, buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        
        //

        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}