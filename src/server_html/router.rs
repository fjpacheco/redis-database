use std::io::Write;

use crate::server_html::{handler::{Css, Handler, StaticPageHandler}, http_response::HttpResponse};
use crate::server_html::page_content::get_page_content;
use crate::server_html::error::http_error::HttpError;

use super::{handler::PageNotFoundHandler/*http_request::{self, HttpRequest}*/};

use crate::server_html::request::{
    http_request::HttpRequest,
    http_method::HttpMethod,
    http_url::HttpUrl,
};

pub struct Router;
impl Router {
    pub fn route(req: HttpRequest, stream: &mut impl Write) -> () {
        let method = req.get_method(); 
        match method {
            // If GET request
            HttpMethod::Get => Router::do_get(req, stream).unwrap(),
            //HttpMethod::Post => Router::do_post(req, stream).unwrap(),
            // If method is not GET request, return 404 page
            _ => {
                let resp: HttpResponse = PageNotFoundHandler::handle(&req);
                let _ = resp.send_response(stream);
            }
        }
    }

    fn do_get(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        match req.get_url() {
            HttpUrl::Path(s) => Router::process_get_url(s, req, stream)
        }
    }

    /*fn do_post(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        match req.get_url() {
            HttpUrl::Path(s) => Router::process_post_url(s, req, stream)
        }
    }*/

    fn process_get_url(s: String, req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        // Parse the URI
        let command= s.split("=").collect::<Vec<&str>>();
        println!("[Router::Get_Request]: resource received: {:?} => {:?}", s,command);
        match command[0] {
            "/try-redis-500x50.jpg" => Router::load_try_redis_logo(req, stream),
            "/style.css" => Router::load_style_css(req, stream),
            "/?command" => Router::process_command(req, stream),
            // Else, invoke static page handler
            _ => {
                let resp: HttpResponse = StaticPageHandler::handle(&req);
                let _ = resp.send_response(stream);
                Ok(())
            }
        }
    }

    /*fn process_post_url(s: String, req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        // Parse the URI
        let command= s.split("=").collect::<Vec<&str>>();
        println!("[Router::Get_Request]: resource received: {:?} => {:?}", s,command);
        match command[0] {                
            "/?command" => Router::process_command(req, stream),
            // Else, invoke static page handler
            _ => {
                let resp: HttpResponse = StaticPageHandler::handle(&req);
                let _ = resp.send_response(stream);
                Ok(())
            }
        }
    }*/

    fn load_try_redis_logo(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        // TODO: ESTO NO LOGRÉ HACERLO FUNCIONAR LPM! NO SE COMO MANDAR UNA IMAGEN CON EL CONTENT-TYPE IMAGE/JPEG

         // TODO: FORMA 1) CROTA QUE NO ANDUVO
         let HttpUrl::Path(s) = req.get_url();

         // Parse the URI
         let uri: Vec<&str> = s.split("=").collect();
         let _command = uri[0].to_string().replace("+", " ");
        
        let contents = "data:image/gif;base64,R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==";     
        let response = format!("HTTP/1.0 200 OK\r\nContent-Type: image/jpeg,\r\nContent-Length: {}\r\n\r\n",
             contents.len(),
         );
         stream.write_all(response.as_bytes()).unwrap();
         stream.write_all(contents.as_bytes()).unwrap();
         stream.flush().unwrap();

         // TODO: FORMA 2) OP QUE NO ANDUVO: 
         //let resp: HttpResponse = Image::handle(&req);
         //let _ = resp.send_response(stream);

         Ok(())
     }

    fn load_style_css(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        let resp: HttpResponse = Css::handle(&req);
        let _ = resp.send_response(stream);
        Ok(())
    }

    fn process_command(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        // TODO: FORMA CROTA
        let HttpUrl::Path(s) = req.get_url();
        // Parse the URI
        let uri: Vec<&str> = s.split("=").collect();
        let command = uri[1].to_string().replace("+", " ");
        let contents = get_page_content(&command);
        let response = format!("HTTP/1.0 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
            contents.len(),
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(contents.as_bytes()).unwrap();
        stream.flush().unwrap();

        // TODO: FORMA OP: 
        /*let resp: HttpResponse = CommandRedis::handle(&req);
        let _ = resp.send_response(stream);*/

        Ok(())
    }
}
