use crate::server_html::{request::http_request::HttpRequest, router::Router};
use std::net::TcpListener;

pub struct ServerHtml<'a> {
    socket_addr: &'a str,
}

impl<'a> ServerHtml<'a> {
    pub fn new(socket_addr: &'a str) -> Self {
        ServerHtml { socket_addr }
    }

    pub fn run(&self) {
        let connection_listener = TcpListener::bind(self.socket_addr).unwrap();
        println!("Running on {}", self.socket_addr);

        for mut stream in connection_listener
            .incoming()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
        {
            println!("Connection established");
            if let Ok(req) = HttpRequest::new(&mut stream) {
                println!("{:?}", req);
                Router::route(req, &mut stream);
            } // TODO: analizar qué error podria dejarse pasar y qué error hay que escribir en html_request....
            println!("Fin Connection");
        }
    }
}
