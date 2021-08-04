use std::{io::Read, net::TcpListener};
use std::io::{BufRead, BufReader};
use crate::server_html::{request::http_request::HttpRequest, router::Router};

pub struct ServerHtml<'a> {
    socket_addr: &'a str,
}

impl<'a> ServerHtml<'a> {
    pub fn new(socket_addr: &'a str) -> Self {
        ServerHtml { socket_addr }
    }
    
    pub fn run(&self) {
        // Start a server listening on socket address
        let connection_listener = TcpListener::bind(self.socket_addr).unwrap();
        println!("Running on {}", self.socket_addr);
        // Listen to incoming connections in a loop
        for stream in connection_listener.incoming() {
            let mut stream = stream.unwrap();
            println!("Connection established");
            let reader = BufReader::new(stream.try_clone().unwrap());
            let mut lines = reader.lines();
            //let first_lecture = lines.next().unwrap();
            // Convert HTTP request to Rust data structure
            //println!("read_buffer: {:?}", read_buffer);
            //let req: HttpRequest = HttpRequest::new(first_lecture.unwrap(), &mut lines).unwrap();
            // Route request to appropriate handler
            //Router::route(req, &mut stream);
            while let Some(first_lecture) = lines.next() {
                let req: HttpRequest = HttpRequest::new(first_lecture.unwrap(), &mut lines).unwrap();
                println!("{:?}", req);
                // Route request to appropriate handler
                Router::route(req, &mut stream);
            }
            println!("Fin Connection");
        }
    }
}
