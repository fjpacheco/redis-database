use std::{collections::HashMap, io::Write};

use crate::server_html::error::http_error::HttpError;
use crate::server_html::status_codes::status_code::StatusCode;

use super::status_codes::status_code;

#[derive(Debug, PartialEq, Clone)]
pub struct HttpResponse {
    version: String,
    status_code: String,
    status_text: String,
    headers: Option<HashMap<String, String>>,
    body: Option<Vec<u8>>,
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            version: "HTTP/1.1".into(),
            status_code: "200".into(),
            status_text: "OK".into(),
            headers: None,
            body: None,
        }
    }
}
impl HttpResponse {
    pub fn new(
        status_code: StatusCode,
        headers: Option<HashMap<String, String>>,
        body: Option<Vec<u8>>,
    ) -> HttpResponse {
        let mut response: HttpResponse = HttpResponse::default();

        let (code, text) = status_code.take_info();
        response.status_code = code;
        response.status_text = text;

        response.headers = match &headers {
            Some(_h) => headers,
            None => {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "text/html".to_string());
                Some(h)
            }
        };

        response.body = body;
        response
    }

    pub fn send_response(&self, write_stream: &mut impl Write) -> Result<(), HttpError> {
        write_stream
            .write_all(&self.as_bytes())
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        write_stream
            .flush()
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        Ok(())
    }
}
impl HttpResponse {
    fn version(&self) -> String {
        self.version.clone()
    }

    fn status_code(&self) -> String {
        self.status_code.clone()
    }

    fn status_text(&self) -> String {
        self.status_text.clone()
    }

    fn headers(&self) -> String {
        let mut header_string: String = "".into();
        if self.headers.is_some() {
            let map: HashMap<String, String> = self.headers.clone().unwrap();
            for (k, v) in map.iter() {
                header_string = format!("{}{}:{}\r\n", header_string, k, v);
            }
        }
        header_string
    }

    pub fn body_bytes(&self) -> &Option<Vec<u8>> {
        &self.body
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut response = format!(
            "{} {} {}\r\n{}Content-Length: {}\r\n\r\n",
            &self.version(),
            &self.status_code(),
            &self.status_text(),
            &self.headers(),
            &self.body_bytes().as_ref().unwrap_or(&Vec::new()).len(),
        )
        .into_bytes();

        response.extend(self.body_bytes().as_ref().unwrap_or(&Vec::new()));

        response
    }
}

impl From<HttpError> for HttpResponse {
    fn from(err: HttpError) -> HttpResponse {
        let (code, description) = err.take();
        Self {
            version: "HTTP/1.1".into(),
            status_code: code,
            status_text: description,
            headers: None,
            body: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_html::status_codes::status_code;
    #[test]
    fn test_response_struct_creation_200() {
        let response_actual = HttpResponse::new(
            status_code::defaults::ok(),
            None,
            Some("I'm Bad Body!".into()),
        );

        let response_expected = HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: "200".to_string(),
            status_text: "OK".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "text/html".to_string());
                Some(h)
            },
            body: Some("I'm Bad Body!".into()),
        };

        assert_eq!(response_actual, response_expected);
    }
    #[test]
    fn test_response_struct_creation_404() {
        let response_actual = HttpResponse::new(
            status_code::defaults::not_found(),
            None,
            Some("I'm Bad Body 404!".into()),
        );
        let response_expected = HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: "404".to_string(),
            status_text: "Page not found".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "text/html".to_string());
                Some(h)
            },
            body: Some("I'm Bad Body 404!".into()),
        };

        assert_eq!(response_actual, response_expected);
    }
    #[test]
    fn test_http_response_creation() {
        let response_expected = HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: "404".to_string(),
            status_text: "Not Found".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "text/html".to_string());
                Some(h)
            },
            body: Some("I'm Bad Body!".into()),
        };

        let http_string = response_expected.as_bytes();

        let response_actual = "HTTP/1.1 404 Not Found\r\nContent-Type:text/html\r\nContent-Length: 13\r\n\r\nI'm Bad Body!".as_bytes();
        assert_eq!(http_string, response_actual);
    }
}
