use crate::server_html::error::http_error::HttpError;
use crate::server_html::request::{http_method::HttpMethod, http_url::HttpUrl};
use crate::server_html::status_codes::status_code::defaults;
use std::collections::HashMap;
use std::convert::From;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: HttpMethod,
    url: HttpUrl,
    http_version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpRequest {
    pub fn new(new_request: &mut dyn Read) -> Result<HttpRequest, HttpError> {
        let mut buf_reader = BufReader::new(new_request);
        let http_first_line = get_req_line(&mut buf_reader)?;
        let (method, url, http_version) = process_req_line(http_first_line)?;
        let headers = get_headers(&mut buf_reader)?;
        let body = get_body(&headers, buf_reader)?;

        Ok(HttpRequest {
            method,
            url,
            http_version,
            headers,
            body,
        })
    }

    pub fn get_method(&self) -> HttpMethod {
        self.method.clone()
    }

    pub fn get_url(&self) -> HttpUrl {
        self.url.clone()
    }

    pub fn get_body(&self) -> Option<&String> {
        self.body.as_ref()
    }
}

fn get_body(
    headers: &HashMap<String, String>,
    mut buf_reader: BufReader<&mut dyn Read>,
) -> Result<Option<String>, HttpError> {
    let mut body = None;
    if let Some(body_size) = headers.get("Content-Length") {
        let buffer = buf_reader
            .fill_buf()
            .map_err(|_| HttpError::new(defaults::bad_request()))?;
        let body_string = String::from_utf8_lossy(&buffer).to_string();
        let length = buffer.len();
        buf_reader.consume(length);

        if body_size
            .parse::<usize>()
            .map_err(|_| HttpError::new(defaults::bad_request()))?
            > body_string.len()
        {
            return Err(HttpError::new(defaults::request_entity_too_large()));
        }
        body = Some(body_string);
    }
    Ok(body)
}

fn get_req_line(buf_reader: &mut BufReader<&mut dyn Read>) -> Result<String, HttpError> {
    let mut http_first_line = String::new();
    let n = buf_reader
        .read_line(&mut http_first_line)
        .map_err(|_| HttpError::new(defaults::bad_request()))?;
    if n.eq(&0) {
        return Err(HttpError::new(defaults::bad_request()));
    }
    Ok(http_first_line)
}

fn get_headers(
    buf_reader: &mut BufReader<&mut dyn Read>,
) -> Result<HashMap<String, String>, HttpError> {
    let mut headers = HashMap::new();
    for packed_new_header in buf_reader.by_ref().lines() {
        if let Ok(new_header) = packed_new_header {
            if new_header.is_empty() {
                break;
            } else {
                let mut tuple: Vec<String> = new_header.split(": ").map(String::from).collect();
                let field_value = match tuple.pop() {
                    Some(version) => version,
                    None => return Err(HttpError::new(defaults::bad_request())),
                };

                let header_field_name = match tuple.pop() {
                    Some(version) => version,
                    None => return Err(HttpError::new(defaults::bad_request())),
                };

                headers.insert(header_field_name, field_value);
            }
        } else {
            return Err(HttpError::new(defaults::bad_request()));
        }
    }
    Ok(headers)
}

fn process_req_line(req_line: String) -> Result<(HttpMethod, HttpUrl, String), HttpError> {
    let mut parsed_first_line: Vec<String> =
        req_line.split_whitespace().map(String::from).collect();

    // Ver como modularizar estos matchs

    let http_version = match parsed_first_line.pop() {
        Some(version) => version,
        None => return Err(HttpError::new(defaults::bad_request())),
    };
    let url = HttpUrl::Path(match parsed_first_line.pop() {
        Some(url) => url,
        None => return Err(HttpError::new(defaults::bad_request())),
    });
    let method = HttpMethod::from(match parsed_first_line.pop() {
        Some(method) => method,
        None => return Err(HttpError::new(defaults::bad_request())),
    })?;

    // Fin

    Ok((method, url, http_version))
}

#[cfg(test)]
pub mod test_http_request {

    use super::*;
    #[test]
    fn test_01() {
        let emuled_request: String =
            String::from("GET /greeting HTTP/1.1\r\nPort: 8080\r\nRust-eze: Team\r\n\r\n");
        let mut headers_expected: HashMap<String, String> = HashMap::new();
        headers_expected.insert("Port".into(), "8080".into());
        headers_expected.insert("Rust-eze".into(), "Team".into());

        let request_parsed = HttpRequest::new(&mut emuled_request.as_bytes()).unwrap();

        assert_eq!(headers_expected, request_parsed.headers);
    }

    #[test]
    fn test_02() {
        let body = "I'm a Body!".to_string();
        let emuled_request: String = format!("POST /greeting HTTP/1.1\r\nPort: 8080\r\nRust-eze: Team\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
        let mut headers_expected: HashMap<String, String> = HashMap::new();
        headers_expected.insert("Port".into(), "8080".into());
        headers_expected.insert("Rust-eze".into(), "Team".into());
        headers_expected.insert("Content-Length".into(), body.len().to_string());

        let request_parsed = HttpRequest::new(&mut emuled_request.as_bytes()).unwrap();

        assert_eq!(headers_expected, request_parsed.headers);
        assert_eq!(Some(body), request_parsed.body);
    }

    #[test]
    fn test_03() {
        let large_body = String::from_utf8(vec![b'X'; 8192]).unwrap();
        let emuled_request: String = format!("POST /greeting HTTP/1.1\r\nPort: 8080\r\nRust-eze: Team\r\nContent-Length: {}\r\n\r\n{}", large_body.len(), large_body);
        let mut headers_expected: HashMap<String, String> = HashMap::new();
        headers_expected.insert("Port".into(), "8080".into());
        headers_expected.insert("Rust-eze".into(), "Team".into());
        headers_expected.insert("Content-Length".into(), large_body.len().to_string());

        let request_parsed = HttpRequest::new(&mut emuled_request.as_bytes());

        assert_eq!(
            request_parsed.unwrap_err(),
            HttpError::new(defaults::request_entity_too_large())
        );
    }
}
