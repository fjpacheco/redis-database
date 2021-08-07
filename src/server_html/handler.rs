use std::fs::File;
use std::io::{Read, Write};
use std::{collections::HashMap, fs};

use super::error::http_error::HttpError;
use super::http_response::HttpResponse;
use crate::server_html::page_content::get_page_content;
use crate::server_html::request::{http_request::HttpRequest, http_url::HttpUrl};
use crate::server_html::status_codes::status_code;

pub trait Handler {
    fn handle(req: &HttpRequest) -> HttpResponse;
    fn load_file(file_name: &str) -> Option<String> {
        let contents = fs::read_to_string(file_name);
        contents.ok()
    }
}

pub struct Css;

impl Handler for Css {
    fn handle(_req: &HttpRequest) -> HttpResponse {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("Content-Type".to_string(), "text/css".to_string());
        HttpResponse::new(
            status_code::defaults::ok(),
            Some(map),
            Self::load_file("src/server_html/resource/style.css"),
        )
    }
}

pub struct Image;

impl Handler for Image {
    fn handle(_req: &HttpRequest) -> HttpResponse {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("Content-Type".to_string(), "image/jpeg".to_string());
        HttpResponse::new(
            status_code::defaults::ok(),
            Some(map),
            Self::load_file("src/server_html/resource/image.html"),
        )
    }
}

pub struct ImagePng;

impl ImagePng {
    /// Recordar los map_err() :>
    pub fn send_image(file_name: &str, stream: &mut impl Write) -> Result<(), HttpError> {
        let file_path = format!("src/server_html/resource/{}", file_name);

        let mut buff_image = Vec::new();
        let mut file = File::open(&file_path)
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        file.read_to_end(&mut buff_image)
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        file.flush()
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;

        let content_length = format!("Content-Length: {}", buff_image.len());

        let headers = [
            "HTTP/1.1 200 OK",
            "Content-type: image/png",
            &content_length,
            "\r\n",
        ];

        let mut response = headers.join("\r\n").into_bytes();
        response.extend(buff_image);

        stream.write_all(&response).unwrap();
        stream.flush().unwrap();
        stream.write_all(b"\r\n").unwrap();
        stream.flush().unwrap();
        Ok(())
    }
}

pub struct CommandRedis;

impl Handler for CommandRedis {
    fn handle(req: &HttpRequest) -> HttpResponse {
        /*// Get the path of static page resource being requested
        let HttpUrl::Path(s) = req.get_url();

        // Parse the URI
        let uri: Vec<&str> = s.split("=").collect();
        let command = uri[1].to_string().replace("+", " ");

        let body = Some(get_page_content(&command));

        HttpResponse::new(status_code::defaults::ok(), None, body)*/

        let default_command = String::from("empty!");
        let uri: Vec<&str> = req
            .get_body()
            .unwrap_or(&default_command)
            .split('=')
            .collect();

        let command = uri[1].to_string().replace("+", " ");
        // ACA VA LO DE MARTO

        let contents = get_page_content(&command);

        HttpResponse::new(status_code::defaults::ok(), None, Some(contents))
    }
}

pub struct PageNotFoundHandler;

impl Handler for PageNotFoundHandler {
    fn handle(_req: &HttpRequest) -> HttpResponse {
        HttpResponse::new(
            status_code::defaults::not_found(),
            None,
            Self::load_file("src/server_html/resource/404.html"),
        )
    }
}
pub struct StaticPageHandler;

impl Handler for StaticPageHandler {
    fn handle(req: &HttpRequest) -> HttpResponse {
        // Get the path of static page resource being requested
        let HttpUrl::Path(s) = req.get_url();

        // Parse the URI
        let route: Vec<&str> = s.split('/').collect();
        match route[1] {
            "" => HttpResponse::new(
                status_code::defaults::ok(),
                None,
                Self::load_file("src/server_html/resource/index.html"),
            ),
            path => match Self::load_file(path) {
                Some(contents) => {
                    let mut map: HashMap<String, String> = HashMap::new();
                    if path.ends_with(".css") {
                        map.insert("Content-Type".to_string(), "text/css".to_string());
                    } else if path.ends_with(".js") {
                        map.insert("Content-Type".to_string(), "text/javascript".to_string());
                    } else {
                        map.insert("Content-Type".to_string(), "text/html".to_string());
                    }
                    HttpResponse::new(status_code::defaults::ok(), Some(map), Some(contents))
                }
                None => HttpResponse::new(
                    status_code::defaults::not_found(),
                    None,
                    Self::load_file("src/server_html/resource/404.html"),
                ),
            },
        }
    }
}
