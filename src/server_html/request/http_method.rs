use crate::server_html::error::http_error::HttpError;
use crate::server_html::status_codes::client_error_code::ClientErrorCode;
use crate::server_html::status_codes::status_code::StatusCode;

/// HttpMethod encapsules the methods that our html code can request
#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
}

impl HttpMethod {
    /// Get the string representation of the enum 'POST' or 'GET' method. 
    ///
    /// # Error
    /// Return an [HttpError] if:
    ///
    /// * The enum is not 'GET' or 'POST'.
    pub fn from(method_str: String) -> Result<HttpMethod, HttpError> {
        match method_str.as_ref() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            _ => Err(HttpError::from(StatusCode::ClientError(
                ClientErrorCode::BadRequest(
                    "The request could not be understood by the server due to malformed
                syntax."
                        .to_string(),
                ),
            ))),
        }
    }
}

#[cfg(test)]
pub mod test_http_methoid {

    use super::*;
    #[test]
    fn test_01_from_get_method() {
        assert_eq!(HttpMethod::Get, HttpMethod::from("GET".to_string()).unwrap());
    }

    #[test]
    fn test_02_from_set_method () {
        assert_eq!(HttpMethod::Post, HttpMethod::from("POST".to_string()).unwrap());
    }

    #[test]
    fn test_03_return_err_if_there_is_no_method() {
        assert!(HttpMethod::from("X".to_string()).is_err())
    }

}