use std::io::Write;

use crate::server_html::error::http_error::HttpError;
use crate::server_html::request::http_method::HttpMethod;
use crate::server_html::{
    handler::{CommandRedisPage, Handler, StaticPage},
    http_response::HttpResponse,
};

use crate::server_html::request::{http_request::HttpRequest, http_url::HttpUrl};

use super::status_codes::status_code;

pub struct Router;
impl Router {
    pub fn route(req: HttpRequest, stream: &mut impl Write) -> Result<(), HttpError> {
        match req.get_url() {
            HttpUrl::Path(url) => match url.as_str() {
                "/?command" => Router::process_command(req, stream)?,
                _ => {
                    let resp: HttpResponse = StaticPage::handle(&req)?;
                    resp.send_response(stream)?;
                }
            },
        }

        Ok(())
    }

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

    use std::{fs::File, io::Read};

    use super::*;
    #[test]
    fn test_01_response_command_in_set_key_value_post_method() {
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

        assert!(response_stream_string.contains("set key value")) // TODO: Ojo, con lo de martu esto será diferente!
    }

    #[test]
    fn test_02_response_command_in_set_key_value_post_method() {
        let body = "".to_string();
        let emuled_request: String = format!(
            "GET / HTTP/1.1\r\nPort: 8080\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let request_parsed = HttpRequest::new(&mut emuled_request.as_bytes()).unwrap();

        let mut response_stream = vec![];
        let _ = Router::route(request_parsed, &mut response_stream);
        let response_stream_string = String::from_utf8_lossy(&response_stream).to_string();

        let file_name = String::from("src/server_html/resource/index.html");
        let mut buff_image = Vec::new();
        let mut file = File::open(&file_name).unwrap();
        file.read_to_end(&mut buff_image).unwrap();
        file.flush().unwrap();
        let response_stream_string_expected = String::from_utf8_lossy(&response_stream).to_string();

        assert!(response_stream_string.contains(&response_stream_string_expected))
    }
}
