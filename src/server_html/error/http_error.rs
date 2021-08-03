use crate::server_html::status_codes::status_code::StatusCode;

#[derive(Debug, Clone)]
pub struct HttpError {
    status_code: StatusCode,
}

impl HttpError {
    pub fn new(status_code: StatusCode) -> HttpError {
        HttpError { status_code }
    }
}
