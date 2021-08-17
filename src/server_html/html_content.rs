use std::{
    fs::{self, OpenOptions},
    io::Write,
};
/// Generates in a string the html code with a response box provided by redis database.

pub fn get_page_content(command: String, redis_response: &str) -> String {
    let response_content = format!("<div><div class=\"line input\"><div class=\"nopad\"><span class=\"prompt\">&gt; </span><a href=\"#run\">{}</a></div></div><div class=\"line error\"><div class=\"nopad\"><span class=\"prompt\"></span>{}</div></div></div>", command, redis_response);
    let mut file = match OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .append(true)
        .open(
            "/Users/martinaagata/Desktop/Rust-eze/src/server_html/resource/top_content".to_string(),
        ) {
        Ok(file) => file,
        Err(err) => return format!("Opening file failed. Detail: {}", err),
    };

    file.write_all(response_content.as_bytes()).unwrap(); // TODO

    let mut content = fs::read_to_string(
        "/Users/martinaagata/Desktop/Rust-eze/src/server_html/resource/top_content".to_string(),
    )
    .unwrap(); // TODO?
    let bottom_content = fs::read_to_string(
        "/Users/martinaagata/Desktop/Rust-eze/src/server_html/resource/bottom_content".to_string(),
    )
    .unwrap(); // TODO?

    content.push_str(&bottom_content);

    content
}

/// Generates the html code in a string for the error page, according to the received code and description.
pub fn get_page_content_error((code, description): (String, String)) -> String {
    format!(
        "
        <html>
            <head>
                <meta charset=\"utf-8\" />
                <title>{} Error - Rust-eze</title>
                <link href=\"favicon.png\" rel=\"shortcut icon\">
                <link href=\"/style_error.css\" rel=\"stylesheet\">

            </head>

            <body>
                <div id=\"logo-rust-eze\">
                    <img src=\"logo-black.png\">
                </div>

                <div id=\"title\">
                    <h1>{} - Error</h1>
                </div>

                <div id=\"description\">
                    <p>{}</p>
                </div>

            </body>
        </html>",
        code, code, description
    )
}
