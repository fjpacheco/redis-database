use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

use crate::server_html::{
    error::http_error::HttpError,
    http_response::HttpResponse,
    request::{http_method::HttpMethod, http_request::HttpRequest, http_url::HttpUrl},
    status_codes::status_code,
};

use super::handler_page::HandlerPage;

pub struct StaticPage;

impl HandlerPage for StaticPage {
    /// It handles static pages that only need to be inserted into the body of an [HttpRespone](crate::server_html::http_response::HttpResponse)
    /// without other complex interaction like PNG images, plain text, and cascading style sheets.
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError> {
        if req.get_method() != &HttpMethod::Get {
            return Err(HttpError::from(status_code::defaults::bad_request()));
        }

        let HttpUrl::Path(s) = req.get_url();

        let route: Vec<&str> = s.split('/').collect();
        match route[1] {
            "" => {
                let base_top_content =
                    fs::read_to_string("src/server_html/resource/base_top_content".to_string())
                        .unwrap(); // TODO?
                let mut file =
                    File::create("src/server_html/resource/top_content".to_string()).unwrap();
                file.write_all(base_top_content.as_bytes()).unwrap(); // TODO

                return Ok(HttpResponse::new(
                    status_code::defaults::ok(),
                    None,
                    Self::load_file("index.html")?,
                ));
            }
            "?clean" => {
                let base_top_content =
                    fs::read_to_string("src/server_html/resource/base_top_content".to_string())
                        .unwrap(); // TODO?
                let mut file =
                    File::create("src/server_html/resource/top_content".to_string()).unwrap();
                file.write_all(base_top_content.as_bytes()).unwrap(); // TODO

                return Ok(HttpResponse::new(
                    status_code::defaults::ok(),
                    None,
                    Self::load_file("index.html")?,
                ));
            }
            path => {
                let mut map: HashMap<String, String> = HashMap::new();
                let mut split_path: Vec<&str> = path.split('.').collect();

                match split_path.pop().unwrap() {
                    "css" => {
                        map.insert("Content-Type".to_string(), "text/css".to_string());
                    }
                    "png" => {
                        map.insert("Content-Type".to_string(), "image/png".to_string());
                    }
                    "html" => {
                        map.insert("Content-Type".to_string(), "text/html".to_string());
                    }
                    _ => {
                        return Err(HttpError::from(status_code::defaults::bad_request()));
                    }
                }

                Ok(HttpResponse::new(
                    status_code::defaults::ok(),
                    Some(map),
                    Self::load_file(path)?,
                ))
            }
        }
    }
}
