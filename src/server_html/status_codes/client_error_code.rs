use std::string::ToString;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientErrorCode {
    BadRequest(String),
    RequestEntityTooLarge(String),
    NotFound(String),
    LengthRequired(String),
}

impl ToString for ClientErrorCode {
    fn to_string(&self) -> String {
        match self {
            ClientErrorCode::BadRequest(description) => format!("{} {}", 400, description),
            ClientErrorCode::RequestEntityTooLarge(description) => {
                format!("{} {}", 413, description)
            }
            ClientErrorCode::NotFound(description) => format!("{} {}", 404, description),
            ClientErrorCode::LengthRequired(description) => format!("{} {}", 411, description),
        }
    }
}
