use crate::server_html::status_codes::client_error_code::ClientErrorCode;
use crate::server_html::status_codes::successfull_code::SuccessfullCode;
use std::string::ToString;

#[derive(Debug, Clone)]
pub enum StatusCode {
    Informational(u16, String),
    Successfull(SuccessfullCode),
    Redirection(u16, String),
    ClientError(ClientErrorCode),
    ServerError(u16, String),
}

impl ToString for StatusCode {
    fn to_string(&self) -> String {
        match self {
            StatusCode::Informational(code, description) => format!("{} {}", code, description),
            StatusCode::Successfull(code) => code.to_string(),
            StatusCode::Redirection(code, description) => format!("{} {}", code, description),
            StatusCode::ClientError(code) => code.to_string(),
            StatusCode::ServerError(code, description) => format!("{} {}", code, description),
        }
    }
}

pub mod defaults {

    use super::*;

    pub fn ok() -> StatusCode {
        StatusCode::Successfull(SuccessfullCode::Ok("OK".to_string()))
    }

    pub fn bad_request() -> StatusCode {
        StatusCode::ClientError(ClientErrorCode::BadRequest(
            "The request could not be understood by the server due to malformed
            syntax."
                .to_string(),
        ))
    }

    pub fn not_found() -> StatusCode {
        StatusCode::ClientError(ClientErrorCode::NotFound(
            "The request could not be understood by the server due to malformed
            syntax."
                .to_string(),
        ))
    }

    pub fn length_required() -> StatusCode {
        StatusCode::ClientError(ClientErrorCode::LengthRequired(
            "Server refuses to accept the request without a defined Content-Length".to_string(),
        ))
    }
}
