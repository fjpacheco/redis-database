use redis_rust::server_html::server_html::ServerHtml;

fn main() -> Result<(), std::io::Error> {
    let server = ServerHtml::new("localhost:8080".to_string());
    server.run()?;
    Ok(())
}
