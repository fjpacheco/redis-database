use crate::native_types::ErrorStruct;

pub mod _type;
pub mod copy;
pub mod del;
pub mod rename;
//pub mod exists;
pub mod clean;

fn pop_value(buffer: &mut Vec<&str>, name: &str) -> Result<String, ErrorStruct> {
    if let Some(value) = buffer.pop() {
        Ok(String::from(value))
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            "wrong number of arguments for ".to_owned() + "\'" + name + "\'" + " command",
        ))
    }
}

fn no_more_values(buffer: &[&str], name: &str) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        Ok(())
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            "wrong number of arguments for ".to_owned() + "\'" + name + "\'" + " command",
        ))
    }
}

fn parse_integer(value: String) -> Result<isize, ErrorStruct> {
    if let Ok(index) = value.parse::<isize>() {
        Ok(index)
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("value is not an integer or out of range"),
        ))
    }
}
