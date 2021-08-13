use crate::server_html::error::http_error::HttpError;
use crate::server_html::status_codes::client_error_code::ClientErrorCode;
use crate::server_html::status_codes::status_code::StatusCode;

#[derive(Debug, PartialEq)]
/// HttpMethod encapsules the methods that our html code can request
pub enum HttpMethod {
    Get,
    Post,
}

impl HttpMethod {
    pub fn from(method_str: String) -> Result<HttpMethod, HttpError> {
        match method_str.as_ref() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            _ => Err(HttpError::new(StatusCode::ClientError(
                ClientErrorCode::BadRequest(
                    "The request could not be understood by the server due to malformed
                syntax."
                        .to_string(),
                ),
            ))),
        }
    }
}
