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
        "COMMAND" => Ok("(error) I'm sorry, I don't recognize that command. Please
        type HELP for one of these commands: DECRBY, DEL, EXISTS, GET, GETSET, INCRBY,
        KEYS, LINDEX, LLEN, LPOP, LPUSH, LRANGE, LREM, LSET, MGET, MSET, RENAME, RPOP,
        RPUSH, SADD, SCARD, SET, SISMEMBER, SMEMBERS, SORT, SREM, TTL, TYPE"
            .to_string()),
        _ => Err(HttpError::from(
            status_code::defaults::internal_server_error(),
        )),
    }
}
