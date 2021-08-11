use crate::server_html::status_codes::status_code::StatusCode;
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpError {
    status_code: StatusCode,
}

impl HttpError {
    pub fn new(status_code: StatusCode) -> HttpError {
        HttpError { status_code }
    }

    pub fn take(self) -> (String, String) {
        let mut code = self.status_code.to_string();
        let mut description = code.split_off(3);
        description.remove(0);
        (code, description)
    }

    pub fn get_status_code(&self) -> StatusCode {
        self.status_code.clone()
    }
}

impl From<StatusCode> for HttpError {
    fn from(status_code: StatusCode) -> HttpError {
        HttpError { status_code }
    }
}
