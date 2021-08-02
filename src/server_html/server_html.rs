use std::{io::Read, net::TcpListener};

use crate::server_html::{http_request::HttpRequest, router::Router};

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
            let mut read_buffer = [0; 90];
            stream.read(&mut read_buffer).unwrap();
            // Convert HTTP request to Rust data structure
            println!("read_buffer: {:?}", read_buffer);
            let req: HttpRequest = String::from_utf8(read_buffer.to_vec()).unwrap().into();
            // Route request to appropriate handler
            Router::route(req, &mut stream);
            println!("Fin Connection");
        }
    }
}
