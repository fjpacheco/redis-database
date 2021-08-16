use crate::server_html::status_codes::status_code::StatusCode;
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Eq)]

/// HttpError defines an error to be displayed as a reply
/// to an interrupted request.
pub struct HttpError {
    status_code: StatusCode,
}

impl HttpError {
    /// # Return value
    /// Returns the code and description (as a tuple) of the contained error
    pub fn take(self) -> (String, String) {
        let mut code = self.status_code.to_string();
        let mut description = code.split_off(3);
        description.remove(0);
        (code, description)
    }

    /// Returns the status code of the contained error.
    pub fn get_status_code(&self) -> &StatusCode {
        &self.status_code
    }
}

impl From<StatusCode> for HttpError {
    /// Creates an HttpError from a give status code.
    fn from(status_code: StatusCode) -> HttpError {
        HttpError { status_code }
    }
}


#[cfg(test)]
pub mod test_http_error {

    use crate::server_html::status_codes::status_code::defaults;

    use super::*;
    #[test]
    fn test_01_from_and_take_test() {
        let error = HttpError::from(defaults::bad_request());
        let (code, description) = error.take();
        assert_eq!(code, "400");
        assert_eq!(description, "Bad request");
    }

    #[test]
    fn test_02_get_status_code() {
        let error = HttpError::from(defaults::not_found());
        let status_code = error.get_status_code();
        assert_eq!(*status_code, defaults::not_found());
    }
}