#[derive(Debug, Clone)]
pub enum SuccessfullCode {
    Ok(String),
}

impl ToString for SuccessfullCode {
    fn to_string(&self) -> String {
        match self {
            SuccessfullCode::Ok(description) => format!("{} {}", 200, description),
        }
    }
}
