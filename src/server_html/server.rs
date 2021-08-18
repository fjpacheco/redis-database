use crate::server_html::html_content::get_page_content_error;
use crate::server_html::http_response::HttpResponse;
use crate::server_html::{request::http_request::HttpRequest, router::Router};
use std::net::TcpListener;

use super::thread_pool::ThreadPool;
pub struct ServerHtml;

impl ServerHtml {
    /// # Start of Client Web for Server Redis
    /// Starts the web server that receives requests from browsers, communicating with them through
    /// the HTTP/1.1 protocol. The description of this protocol is the one corresponding to RFC 2616.
    ///
    /// The server must listen for HTTP requests on TCP port 8080 and will communicate with the [ServerRedis](crate::tcp_protocol::server::ServerRedis)
    /// developed from the implemented redis protocol.
    ///
    /// # Error
    /// Return an [Error](std::io::Error) if:
    ///
    /// * The server cannot be started on the default port (8080).
    /// * The connection was aborted (terminated) by the server.
    pub fn start(socket_addr: &str) -> Result<(), std::io::Error> {
        let connection_listener = TcpListener::bind(socket_addr.to_string())?;
        println!("Running on {}", socket_addr);

        let pool = ThreadPool::new(5);

        for mut stream in connection_listener
            .incoming()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
        {
            pool.spawn(move || match HttpRequest::new(&mut stream) {
                Ok(req) => {
                    if let Err(err) = Router::route(req, &mut stream) {
                        let _ = process_err(err, &mut stream);
                    }
                }
                Err(err) => {
                    let _ = process_err(err, &mut stream);
                }
            });
        }
        Ok(())
    }
}

fn process_err(
    err: super::error::http_error::HttpError,
    stream: &mut std::net::TcpStream,
) -> Result<(), std::io::Error> {
    let code = err.get_status_code().clone();
    let body = Some(get_page_content_error(err.take()).into_bytes());
    let response = HttpResponse::new(code, None, body);
    // We should close the programme if the socket is disconnected
    response.send_response(stream).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            "Http Server has been closed",
        )
    })
}
