use std::string::ToString;
#[derive(Debug, Clone)]
pub enum ClientErrorCode {
    BadRequest(String),
    NotFound(String),
    LengthRequired(String),
}

impl ToString for ClientErrorCode {
    fn to_string(&self) -> String {
        match self {
            ClientErrorCode::BadRequest(description) => format!("{} {}", 400, description),
            ClientErrorCode::NotFound(description) => format!("{} {}", 404, description),
            ClientErrorCode::LengthRequired(description) => format!("{} {}", 411, description),
        }
    }
}
