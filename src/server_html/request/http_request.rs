use crate::server_html::error::http_error::HttpError;
use crate::server_html::request::{http_method::HttpMethod, http_url::HttpUrl};
use crate::server_html::status_codes::status_code::defaults;
use std::collections::HashMap;
use std::convert::From;
use std::io::{BufRead, Lines};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: HttpMethod,
    url: HttpUrl,
    http_version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpRequest {
    pub fn new<G>(http_first_line: String, new_request: &mut Lines<G>) -> Result<HttpRequest, HttpError>
    where
        G: BufRead,
    {

        let (method, url, http_version) = process_req_line(http_first_line)?;

        let headers = get_headers(new_request)?;

        let mut body = None;

        if let Some(body_size) = headers.get("Content-Length") {
            let size = usize::from_str(body_size).map_err(|_| HttpError::new(defaults::length_required()))?;
            let unwrapped_body = get_body(new_request)?;
            if unwrapped_body.len() != size {
                return Err(HttpError::new(defaults::bad_request()))
            }
            body = Some(unwrapped_body);
        } 

        Ok(HttpRequest {method, url, http_version, headers, body})

    }

    pub fn get_method(&self) -> HttpMethod {
        self.method.clone()
    }

    pub fn get_url(&self) -> HttpUrl {
        self.url.clone()
    }
}

fn process_req_line(req_line: String) -> Result<(HttpMethod, HttpUrl, String), HttpError> {
        let mut parsed_first_line: Vec<String> = req_line
            .split_whitespace()
            .map(|slice| String::from(slice))
            .collect();
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

        Ok((method, url, http_version))
}

fn get_headers<G>(new_request: &mut Lines<G>) -> Result<HashMap<String, String>, HttpError>
where
    G: BufRead,
{
    let mut headers = HashMap::new();

    while let Some(packed_new_header) = new_request.next() {
        if let Ok(new_header) = packed_new_header {
            if new_header.is_empty() {
                break;
            } else {
                let mut tuple: Vec<String> = new_header
                    .split(": ")
                    .map(|slice| String::from(slice))
                    .collect();
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

fn get_body<G>(new_request: &mut Lines<G>) -> Result<String, HttpError>
where
    G: BufRead,
    {
        if let Some(packed_new_body) = new_request.next() {
            packed_new_body.map_err(|_| HttpError::new(defaults::bad_request()))
        } else {
            return Err(HttpError::new(defaults::bad_request()));
        }
    }