use std::{collections::HashMap, fs};

use super::http_response::HttpResponse;
use crate::server_html::request::{http_request::HttpRequest, http_url::HttpUrl};
use crate::server_html::page_content::get_page_content;

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
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("Content-Type", "text/css");
        HttpResponse::new("200", Some(map), Self::load_file("src/server_html/resource/style.css"))
    }
}


pub struct Image;

impl Handler for Image {
    fn handle(_req: &HttpRequest) -> HttpResponse {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("Content-Type", "image/jpeg");
        HttpResponse::new("200", Some(map),Self::load_file("src/server_html/resource/image.html"))
    }

}

pub struct CommandRedis;

impl Handler for CommandRedis {
    fn handle(req: &HttpRequest) -> HttpResponse {
        // Get the path of static page resource being requested
        let HttpUrl::Path(s) = req.get_url();

        // Parse the URI
        let uri: Vec<&str> = s.split("=").collect();
        let command = uri[1].to_string().replace("+", " ");
  
        let body = Some(get_page_content(&command));
  
        HttpResponse::new("200",None, body)
     
    }
}


pub struct PageNotFoundHandler;


impl Handler for PageNotFoundHandler {
    fn handle(_req: &HttpRequest) -> HttpResponse {
        HttpResponse::new("404", None, Self::load_file("src/server_html/resource/404.html"))
    }
}
pub struct StaticPageHandler;

impl Handler for StaticPageHandler {
    fn handle(req: &HttpRequest) -> HttpResponse {
        // Get the path of static page resource being requested
        let HttpUrl::Path(s) = req.get_url();

        // Parse the URI
        let route: Vec<&str> = s.split("/").collect();
        match route[1] {
            "" => HttpResponse::new("200", None, Self::load_file("src/server_html/resource/index.html")),
            path => match Self::load_file(path) {
                Some(contents) => {
                    let mut map: HashMap<&str, &str> = HashMap::new();
                    if path.ends_with(".css") {
                        map.insert("Content-Type", "text/css");
                    } else if path.ends_with(".js") {
                        map.insert("Content-Type", "text/javascript");
                    } else {
                        map.insert("Content-Type", "text/html");
                    }
                    HttpResponse::new("200", Some(map), Some(contents))
                }
                None => HttpResponse::new("404", None, Self::load_file("src/server_html/resource/404.html")),
            },
        }
    }
}
