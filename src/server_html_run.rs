use redis_rust::server_html::server::ServerHtml;

fn main() -> Result<(), std::io::Error> {
    ServerHtml::start("localhost:8080")?;
    Ok(())
}
