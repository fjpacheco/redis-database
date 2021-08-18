#[derive(Debug, Clone, PartialEq, Eq)]
/// Enumerates some server error status that the reply
/// could take.
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
