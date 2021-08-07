use std::io::Write;

use crate::server_html::error::http_error::HttpError;
use crate::server_html::{
    handler::{CommandRedis, Css, Handler, ImagePng, StaticPageHandler},
    http_response::HttpResponse,
};

use crate::server_html::request::{
    http_method::HttpMethod, http_request::HttpRequest, http_url::HttpUrl,
};

pub struct Router;
impl Router {
    pub fn route(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        let method = req.get_method();
        match method {
            HttpMethod::Get => Router::do_get(req, stream)?,
            HttpMethod::Post => Router::do_post(req, stream)?,
        }
        Ok(())
    }

    fn do_get(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        match req.get_url() {
            HttpUrl::Path(s) => Router::process_get_url(s, req, stream),
        }
    }

    fn do_post(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        match req.get_url() {
            HttpUrl::Path(s) => Router::process_post_url(s, req, stream),
        }
    }

    fn process_get_url(
        s: String,
        req: HttpRequest,
        stream: &mut impl Write,
    ) -> Result<(), HttpError> {
        let command = s.split('=').collect::<Vec<&str>>();
        match command[0] {
            "/logo-rust-ese-2030.png" => {
                ImagePng::send_image("logo-rust-ese-2030.png", stream)?;
                Ok(())
            }
            "/favicon.png" => {
                ImagePng::send_image("favicon.png", stream)?;
                Ok(())
            }
            "/header-logo.png" => {
                ImagePng::send_image("header-logo.png", stream)?;
                Ok(())
            }
            "/style.css" => Router::load_style_css(req, stream),
            _ => {
                let resp: HttpResponse = StaticPageHandler::handle(&req);
                let _ = resp.send_response(stream);
                Ok(())
            }
        }
    }

    fn process_post_url(
        s: String,
        req: HttpRequest,
        stream: &mut impl Write,
    ) -> Result<(), HttpError> {
        let command = s.split('=').collect::<Vec<&str>>();
        match command[0] {
            "/?command" => Router::process_command(req, stream),
            _ => {
                let resp: HttpResponse = StaticPageHandler::handle(&req);
                let _ = resp.send_response(stream);
                Ok(())
            }
        }
    }

    fn load_style_css(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        let resp: HttpResponse = Css::handle(&req);
        let _ = resp.send_response(stream);
        Ok(())
    }

    fn process_command(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        /*let default_command = String::from("empty!");
        let command: Vec<&str> = req
            .get_body()
            .unwrap_or(&default_command)
            .split('=')
            .collect();
        let contents = get_page_content(command[1]);
        let response = format!(
            "HTTP/1.0 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
            contents.len(),
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(contents.as_bytes()).unwrap();
        stream.flush().unwrap();*/

        let resp: HttpResponse = CommandRedis::handle(&req);
        let _ = resp.send_response(stream);

        println!("[process_command::body]: {:?}", req);

        Ok(())
    }
}
