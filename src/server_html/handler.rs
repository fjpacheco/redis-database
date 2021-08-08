use std::fs::File;
use std::io::{Read, Write};
use std::{collections::HashMap, fs};

use super::error::http_error::HttpError;
use super::http_response::{BodyContent, HttpResponse};
use crate::server_html::page_content::get_page_content;
use crate::server_html::request::{http_request::HttpRequest, http_url::HttpUrl};
use crate::server_html::status_codes::status_code;

pub trait Handler {
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError>;

    fn load_file(file_name: &str) -> Result<BodyContent, HttpError> {
        if file_name.is_empty() {
            return Ok(BodyContent::Empty);
        }

        let file_name = format!("src/server_html/resource/{}", file_name);
        if file_name.ends_with(".png") {
            let mut buff_image = Vec::new();
            let mut file = File::open(&file_name)
                .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
            file.read_to_end(&mut buff_image)
                .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
            file.flush()
                .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
            Ok(BodyContent::Bytes(buff_image))
        } else {
            let contents = fs::read_to_string(file_name);
            Ok(BodyContent::Text(contents.map_err(|_| {
                HttpError::from(status_code::defaults::not_found())
            })?))
        }
    }
}

pub struct CommandRedisPage;

// TODO: para lo de MARTO seguramente acá no deberiamos respstar el trait HAndelr... habrá que pasar channels de alguna manera jeee
impl Handler for CommandRedisPage {
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let default_command = "";
        let command = req
            .get_body()
            .unwrap_or(&default_command.to_string())
            .split('=')
            .collect::<Vec<&str>>()
            .get(1)
            .unwrap_or(&default_command)
            .to_string()
            .replace("+", " ");

        // TODO: ACA VA LO DE MARTO

        let contents = get_page_content(&command);

        Ok(HttpResponse::new(
            status_code::defaults::ok(),
            None,
            BodyContent::Text(contents),
        ))
    }
}

pub struct StaticPage;

impl Handler for StaticPage {
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let HttpUrl::Path(s) = req.get_url();

        let route: Vec<&str> = s.split('/').collect();
        match route[1] {
            "" => Ok(HttpResponse::new(
                status_code::defaults::ok(),
                None,
                Self::load_file("index.html")?,
            )),
            path => match Self::load_file(path)? {
                BodyContent::Text(contents) => process_string_content(path, contents),
                BodyContent::Bytes(contents) => process_bytes_content(path, contents),
                _ => Ok(HttpResponse::new(
                    status_code::defaults::not_found(),
                    None,
                    Self::load_file("404.html")?,
                )),
            },
        }
    }
}

fn process_string_content(path: &str, contents: String) -> Result<HttpResponse, HttpError> {
    let mut map: HashMap<String, String> = HashMap::new();
    if path.ends_with(".css") {
        map.insert("Content-Type".to_string(), "text/css".to_string());
    } else {
        map.insert("Content-Type".to_string(), "text/html".to_string());
    }
    Ok(HttpResponse::new(
        status_code::defaults::ok(),
        Some(map),
        BodyContent::Text(contents),
    ))
}

fn process_bytes_content(path: &str, contents: Vec<u8>) -> Result<HttpResponse, HttpError> {
    let mut map: HashMap<String, String> = HashMap::new();
    if path.ends_with(".png") {
        map.insert("Content-Type".to_string(), "image/png".to_string());
    } else {
        return Ok(HttpResponse::new(
            status_code::defaults::not_found(),
            None,
            StaticPage::load_file("404.html")?,
        ));
    }
    Ok(HttpResponse::new(
        status_code::defaults::ok(),
        Some(map),
        BodyContent::Bytes(contents),
    ))
}
