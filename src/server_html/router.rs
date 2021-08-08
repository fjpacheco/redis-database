use std::io::Write;

use crate::server_html::error::http_error::HttpError;
use crate::server_html::{
    handler::{CommandRedisPage, Handler, StaticPage},
    http_response::HttpResponse,
};

use crate::server_html::request::{http_request::HttpRequest, http_url::HttpUrl};

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
        let resp: HttpResponse = CommandRedisPage::handle(&req)?;
        let _ = resp.send_response(stream);

        Ok(())
    }
}
