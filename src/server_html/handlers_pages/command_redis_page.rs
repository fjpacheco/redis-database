use std::sync::mpsc;

use crate::{joinable::Joinable, native_types::ErrorStruct, server_html::{available_commands::available_commands, error::http_error::HttpError, html_content::get_page_content, http_response::HttpResponse, redis_client::RedisClient, request::http_request::HttpRequest, status_codes::status_code}};

use super::handler_page::HandlerPage;


pub struct CommandRedisPage;

impl HandlerPage for CommandRedisPage {
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let default_command = "";
        let command = req
            .get_body()
            .unwrap_or(&default_command.to_string())
            .split('=')
            .collect::<Vec<&str>>()
            .get(1)
            .unwrap_or(&default_command)
            .to_string()
            .replace("+", " ");

        let response = execute_command(command)?;

        let contents = get_page_content(&response).into_bytes();

        Ok(HttpResponse::new(
            status_code::defaults::ok(),
            None,
            Some(contents),
        ))
    }
}


fn execute_command(command: String) -> Result<String, HttpError> {
    let (cmd_sender, cmd_receiver) = mpsc::channel();
    let (rsp_sender, rsp_receiver) = mpsc::channel();

    let mut client = RedisClient::new(
        available_commands(),
        cmd_sender.clone(),
        rsp_sender,
        cmd_receiver,
        "127.0.0.1:6379".to_string(),
    ).map_err(|err| map_db_err_to_http_err(err))?;

    cmd_sender.send(Some(command))
              .map_err(|_| HttpError::from(
                  status_code::defaults::internal_server_error()
              ))?;

    
    let response = rsp_receiver.recv()
                .map_err(|_| HttpError::from(
                    status_code::defaults::internal_server_error()
                ))?.map_err(|x| x.print_it());

    client.join().map_err(|err| map_db_err_to_http_err(err))?;

    if response.is_err(){
        return Ok(response.unwrap_err())
    }else{
        return Ok(response.unwrap())
    }

}

fn map_db_err_to_http_err(_db_err: ErrorStruct) -> HttpError {
    HttpError::from(
        status_code::defaults::internal_server_error()
    )
}
