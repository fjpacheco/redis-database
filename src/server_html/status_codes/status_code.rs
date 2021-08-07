use crate::server_html::status_codes::client_error_code::ClientErrorCode;
use crate::server_html::status_codes::server_error_code::ServerErrorCode;
use crate::server_html::status_codes::successfull_code::SuccessfullCode;
use std::string::ToString;

#[derive(Debug, Clone)]
pub enum StatusCode {
    Informational(u16, String),
    Successfull(SuccessfullCode),
    Redirection(u16, String),
    ClientError(ClientErrorCode),
    ServerError(ServerErrorCode),
}

impl ToString for StatusCode {
    fn to_string(&self) -> String {
        match self {
            StatusCode::Informational(code, description) => format!("{} {}", code, description),
            StatusCode::Successfull(code) => code.to_string(),
            StatusCode::Redirection(code, description) => format!("{} {}", code, description),
            StatusCode::ClientError(code) => code.to_string(),
            StatusCode::ServerError(code) => code.to_string(),
        }
    }
}

impl StatusCode {
    pub fn take_info(&self) -> (String, String) {
        let mut code = self.to_string();
        let mut description = code.split_off(3);
        description.remove(0);
        (code, description)
    }
}

pub mod defaults {

    use super::*;

    pub fn ok() -> StatusCode {
        StatusCode::Successfull(SuccessfullCode::Ok("OK".to_string()))
    }

    pub fn bad_request() -> StatusCode {
        StatusCode::ClientError(ClientErrorCode::BadRequest("Bad Request".to_string()))
    }

    pub fn not_found() -> StatusCode {
        StatusCode::ClientError(ClientErrorCode::NotFound("Not Found".to_string()))
    }

    pub fn length_required() -> StatusCode {
        StatusCode::ClientError(ClientErrorCode::LengthRequired(
            "Lenght Required".to_string(),
        ))
    }

    pub fn internal_server_error() -> StatusCode {
        StatusCode::ServerError(ServerErrorCode::InternalServerError(
            "Internal Server Error".to_string(),
        ))
    }
}
