use std::sync::mpsc;

use crate::{
    joinable::Joinable,
    native_types::ErrorStruct,
    server_html::{
        available_commands::available_commands, error::http_error::HttpError,
        html_content::get_page_content, http_response::HttpResponse, redis_client::RedisClient,
        request::http_request::HttpRequest, status_codes::status_code,
    },
};

use super::handler_page::HandlerPage;

pub struct CommandRedisPage;

impl HandlerPage for CommandRedisPage {
    /// It is responsible for processing the command received by a user with the redis database.
    /// Provide the answer in [HttpResponse] format.
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

        let response = execute_command(command.clone())?;

        let contents = get_page_content(command, &response).into_bytes();

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
    )
    .map_err(|_| HttpError::from(status_code::defaults::internal_server_error()))?;

    cmd_sender
        .send(Some(command))
        .map_err(|_| HttpError::from(status_code::defaults::internal_server_error()))?;
    let result = rsp_receiver
        .recv()
        .map_err(|_| HttpError::from(status_code::defaults::internal_server_error()))?;
    let response = match result {
        Ok(resp) => resp,
        Err(err) => map_db_err_to_http_response(err)?,
    };

    client
        .join()
        .map_err(|_| HttpError::from(status_code::defaults::internal_server_error()))?;

    Ok(response)
}
// TODO
/*
if response.is_err(){
    return Ok(response.unwrap_err())
}else{
    return Ok(response.unwrap())
}
*/

fn map_db_err_to_http_response(db_err: ErrorStruct) -> Result<String, HttpError> {
    match db_err.prefix().unwrap_or("default") {
        "COMMAND" => Ok(
            "(error) I'm sorry, I don't recognize that command. Commands supported:
        <a href=\"#decrby\">DECRBY</a>,
        <a href=\"#help\">DEL</a>,
        <a href=\"#help\">EXISTS</a>,
        <a href=\"#help\">GET</a>,
        <a href=\"#help\">GETSET</a>,
        <a href=\"#help\">INCRBY</a>,
        <a href=\"#help\">KEYS</a>,
        <a href=\"#help\">LINDEX</a>,
        <a href=\"#help\">LLEN</a>,
        <a href=\"#help\">LPOP</a>,
        <a href=\"#help\">LPUSH</a>,
        <a href=\"#help\">LRANGE</a>,
        <a href=\"#help\">LREM</a>,
        <a href=\"#help\">LSET</a>,
        <a href=\"#help\">MGET</a>,
        <a href=\"#help\">MSET</a>,
        <a href=\"#help\">RENAME</a>,
        <a href=\"#help\">RPOP</a>,
        <a href=\"#help\">RPUSH</a>,
        <a href=\"#help\">SADD</a>,
        <a href=\"#help\">SCARD</a>,
        <a href=\"#help\">SET</a>,
        <a href=\"#help\">SISMEMBER</a>,
        <a href=\"#help\">SMEMBERS</a>,
        <a href=\"#help\">SORT</a>,
        <a href=\"#help\">SREM</a>,
        <a href=\"#help\">TTL</a>,
        <a href=\"#help\">TYPE</a>"
                .to_string(),
        ),
        _ => Err(HttpError::from(
            status_code::defaults::internal_server_error(),
        )),
    }
}
