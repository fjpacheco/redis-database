use redis_rust::server_html::server_html::ServerHtml;

fn main() {
    let server = ServerHtml::new("localhost:8080");
    server.run();
}
