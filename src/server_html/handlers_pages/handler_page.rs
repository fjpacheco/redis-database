use std::{fs::File, io::{Read, Write}};

use crate::server_html::{error::http_error::HttpError, http_response::HttpResponse, request::http_request::HttpRequest, status_codes::status_code};

pub trait HandlerPage {
    /// In charge of handling the [HttpRequest] returning a response in an [HttpResponse].
    /// It can also return an error [HttpError]
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError>;

    /// Returns the content of a file in a <[Option]<[Vec]<[u8]>>.
    /// If the empty file_name is received, it returns a [None]. 
    /// Returns an [HttpError] if there was an error reading the file. 
    fn load_file(file_name: &str) -> Result<Option<Vec<u8>>, HttpError> {
        if file_name.is_empty() {
            return Ok(None);
        }

        let file_name = format!("src/server_html/resource/{}", file_name);
        let mut buff_image = Vec::new();
        let mut file = File::open(&file_name)
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        file.read_to_end(&mut buff_image)
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        file.flush()
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        Ok(Some(buff_image))
    }
}
