#[derive(Debug, Clone, PartialEq, Eq)]
/// Enumerates some successfull status that the reply
/// could take.
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
