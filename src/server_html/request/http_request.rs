use crate::server_html::error::http_error::HttpError;
use crate::server_html::request::{http_method::HttpMethod, http_url::HttpUrl};
use crate::server_html::status_codes::status_code::defaults;
use std::collections::HashMap;
use std::convert::From;
use std::io::{BufRead, BufReader, Lines, Read};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: HttpMethod,
    url: HttpUrl,
    http_version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpRequest {
    pub fn new(new_request: &mut TcpStream) -> Result<HttpRequest, HttpError> {
        let mut buf_reader = BufReader::new(new_request);
        let mut http_first_line = String::new();
        let n = buf_reader
            .read_line(&mut http_first_line)
            .map_err(|_| HttpError::new(defaults::bad_request()))?;
        if n.eq(&0) {
            return Err(HttpError::new(defaults::bad_request())); // TODO: esto esta mal !!!!!!!! puede venir vacio....hay un ejemplo, recordar..
        }
        println!("http_first_line: {:?}", http_first_line);
        let (method, url, http_version) = process_req_line(http_first_line)?;

        let headers = get_headers(&mut buf_reader)?;

        // Encapsular lo siguiente en una funcion

        let mut body = None;

        if let Some(_body_size) = headers.get("Content-Length") {
            // TODO: if body_size > 8192 .......... debatir, poner threshold como tope, sino devolver un ERRROR de numero 40x xd?
            let buffer = buf_reader
                .fill_buf()
                .map_err(|_| HttpError::new(defaults::bad_request()))?;
            body = Some(String::from_utf8_lossy(&buffer).to_string())
        }

        // Fin

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

fn get_headers(
    buf_reader: &mut BufReader<&mut TcpStream>,
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

/*fn get_body<G>(new_request: &mut Lines<G>) -> Result<String, HttpError>
where
    G: BufRead,
{
    if let Some(packed_new_body) = new_request.next() {
        packed_new_body.map_err(|_| HttpError::new(defaults::bad_request()))
    } else {
        return Err(HttpError::new(defaults::bad_request()));
    }
}*/
