use crate::native_types::ErrorStruct;

pub mod channels;
pub mod numsub;
pub mod publish;
pub mod subscribe_cf;
pub mod subscribe_cl;
pub mod unsubscribe_cf;
pub mod unsubscribe_cl;

#[allow(dead_code)]
pub fn pop_value(buffer: &mut Vec<String>, name: &str) -> Result<String, ErrorStruct> {
    if let Some(value) = buffer.pop() {
        Ok(value)
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            "wrong number of arguments for ".to_owned() + "\'" + name + "\'" + " command",
        ))
    }
}

#[allow(dead_code)]
pub fn no_more_values(buffer: &[String], name: &str) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        Ok(())
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            "wrong number of arguments for ".to_owned() + "\'" + name + "\'" + " command",
        ))
    }
}
