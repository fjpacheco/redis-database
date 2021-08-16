/// Generates in a string the html code with a response box provided by redis database.
pub fn get_page_content(redis_response: &str) -> String {
    format!("
    <html>
        <head>
            <title>Try Redis - Rust-eze</title>
            <link href=\"favicon.png\" rel=\"shortcut icon\">
            <link href=\"/style.css\" rel=\"stylesheet\">
        </head>
        <body>
            <div id=\"logo-rust-eze\">
                <img src=\"logo-rust-ese-2030.png\">
            </div>

            <div id=\"header-redis\">
                <img src=\"header-logo.png\">
            </div>
            <div class=\"button-container\">
                <form action=\"?command\" name=\"testForm\" method=\"POST\">
                    <div class=\"from-group\">
                        <div id=\"toolbar\" style=\"display: block;\">
                            <input id=\"input\" spellcheck=\"false\" name=\"command\" class=\"from-control\">
                        </div>
                    </div>
                    <button id=\"button-send\" type=\"submit\" class=\"btn btn-primary\">Enviar</button>
                </form>
            </div>
            <div id=\"footer\">
                <p>Este sitio fue desarrollado por Rust-eze team</p>
                <p><a href=\"https://github.com/taller-1-fiuba-rust/Rust-eze\">Repositorio Github</a></p>
            </div>
            <div id=\"response\">
                <p>{}</p>
            </div>
            </body>
        </html>", redis_response)
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
                    <img src=\"logo-rust-ese-2030.png\">
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
