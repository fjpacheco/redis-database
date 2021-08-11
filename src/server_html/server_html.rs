use crate::server_html::http_response::{BodyContent, HttpResponse};
use crate::server_html::page_content::get_page_content_error;
use crate::server_html::status_codes::status_code;
use crate::server_html::{request::http_request::HttpRequest, router::Router};
use std::net::TcpListener;
pub struct ServerHtml<'a> {
    socket_addr: &'a str,
}

impl<'a> ServerHtml<'a> {
    pub fn new(socket_addr: &'a str) -> Self {
        ServerHtml { socket_addr }
    }

    pub fn run(&self) -> Result<(), std::io::Error> {
        let connection_listener = TcpListener::bind(self.socket_addr)?;
        println!("Running on {}", self.socket_addr);

        for mut stream in connection_listener
            .incoming()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
        {
            println!("Request received");
            match HttpRequest::new(&mut stream) {
                Ok(req) => {
                    if let Err(err) = Router::route(req, &mut stream) {
                        process_err(err, &mut stream)?;
                    }
                }
                Err(err) => {
                    process_err(err, &mut stream)?;
                }
            }
            println!("Response sent");
        }
        Ok(())
    }
}

fn process_err(
    err: super::error::http_error::HttpError,
    stream: &mut std::net::TcpStream,
) -> Result<(), std::io::Error> {
    let body = BodyContent::Text(get_page_content_error(err.take()));
    let response = HttpResponse::new(status_code::defaults::not_found(), None, body);
    // We should close the programme if the socket is disconnected
    response.send_response(stream).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            "Http Server has been closed",
        )
    })
}
