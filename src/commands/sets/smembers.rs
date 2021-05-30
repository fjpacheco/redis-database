use crate::{
    commands::{check_empty_and_name_command, database_mock::Database},
    native_types::ErrorStruct,
};

pub struct Smembers;

impl Smembers {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;
        database.get("none");
        Ok("none".to_string())
    }
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty_and_name_command(&buffer_vec, "smembers")?;

    Ok(())
}

#[cfg(test)]
mod test_smembers_function {
    #[test]
    fn test01() {
        assert_eq!(1 + 1, 2);
    }
}
