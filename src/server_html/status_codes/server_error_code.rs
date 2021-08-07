#[derive(Debug, Clone)]
pub enum ServerErrorCode {
    InternalServerError(String),
}

impl ToString for ServerErrorCode {
    fn to_string(&self) -> String {
        match self {
            ServerErrorCode::InternalServerError(description) => format!("{} {}", 500, description),
        }
    }
}
