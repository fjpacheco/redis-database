use std::io::Write;

use crate::server_html::error::http_error::HttpError;
use crate::server_html::request::http_method::HttpMethod;
use crate::server_html::http_response::HttpResponse;
use crate::server_html::request::{http_request::HttpRequest, http_url::HttpUrl};

use super::handlers_pages::command_redis_page::CommandRedisPage;
use super::handlers_pages::handler_page::HandlerPage;
use super::handlers_pages::static_page::StaticPage;
use super::status_codes::status_code;

pub struct Router;

impl Router {
    /// Structure in charge of handling an [HttpRequest](crate::server_html::request::http_request::HttpRequest) and writes over a stream 
    /// that respects the [Write] function to return a response.
    /// 
    /// In case of respecting the URL surfix '/?Command', will process the command
    /// with the database [ServerRedis](crate::tcp_protocol::server::ServerRedis) implemented, on port 6379.
    ///
    /// # Error
    /// Return a [HttpError](crate::server_html::error::http_error::HttpError) with the status code of the error if:
    ///
    /// - The URL is not a valid command.
    /// - The command is not implemented.
    /// - The stream is closed.
    /// - A command with method GET in html request.
    pub fn route(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        match req.get_url() {
            HttpUrl::Path(url) => match url.as_str() {
                "/?command" => Self::process_command(req, stream)?,
                _ => {
                    let resp: HttpResponse = StaticPage::handle(&req)?;
                    resp.send_response(stream)?;
                }
            },
        }

        Ok(())
    }

    /// Prcocess the command with the database [ServerRedis](crate::tcp_protocol::server::ServerRedis) implemented, on port 6379.
    fn process_command(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        if req.get_method() == &HttpMethod::Post {
            let resp: HttpResponse = CommandRedisPage::handle(&req)?;
            resp.send_response(stream)?;
        } else {
            return Err(HttpError::from(status_code::defaults::bad_request()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, net::TcpListener, thread};

    use super::*;
    #[test]
    #[ignore = "Test with concurrency problem crossing with other tests on the same port in server redis mock"]
    fn long_test_01_response_command_in_set_key_value_post_method() {

        let handler_mock_server = thread::spawn(move || {
            let server_mock = TcpListener::bind("127.0.0.1:6379").unwrap();
            let mut stream_tcp = server_mock.accept().unwrap().0;
            let mut buf = [0; 100];
            let _buffer = stream_tcp.read(&mut buf);
            let _ = stream_tcp.write(b"+OK\r\n");
        });

        let body = "command=set+key+value".to_string();
        let emuled_request: String = format!(
            "POST /?command HTTP/1.1\r\nPort: 8080\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let request_parsed = HttpRequest::new(&mut emuled_request.as_bytes()).unwrap();
        
        let mut response_stream = vec![];
        let _ = Router::route(request_parsed, &mut response_stream);
        let response_stream_string = String::from_utf8_lossy(&response_stream).to_string();
        assert!(response_stream_string.contains("OK")); 

        let _ = handler_mock_server.join();
    }

    #[test]
    fn test_02_response_index_html() {
        // Arrange 
        let body = "".to_string();
        let emuled_request: String = format!(
            "GET / HTTP/1.1\r\nPort: 8080\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let request_parsed = HttpRequest::new(&mut emuled_request.as_bytes()).unwrap();
        let mut response_stream = vec![];
        let file_name = String::from("src/server_html/resource/index.html");
        let mut buff_image = Vec::new();
        let mut file = File::open(&file_name).unwrap();
        file.read_to_end(&mut buff_image).unwrap();
        file.flush().unwrap();
        let response_stream_string_expected = String::from_utf8_lossy(&buff_image).to_string();

        // Act
        let _ = Router::route(request_parsed, &mut response_stream);
        let response_stream_string_received = String::from_utf8_lossy(&response_stream).to_string();

        // Assert
        assert!(response_stream_string_received.contains(&response_stream_string_expected))
    }
}
