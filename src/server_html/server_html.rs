use crate::server_html::http_response::HttpResponse;
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
            println!("Connection established");
            match HttpRequest::new(&mut stream) {
                Ok(req) => {
                    println!("{:?}", req);
                    if let Err(status_code) = Router::route(req, &mut stream) {
                        let response = HttpResponse::from(status_code);
                        let _ = response.send_response(&mut stream);
                    }
                }
                Err(status) => {
                    let response = HttpResponse::from(status);
                    let _ = response.send_response(&mut stream);
                }
            }
            println!("Fin Connection");
        }
        Ok(())
    }
}
