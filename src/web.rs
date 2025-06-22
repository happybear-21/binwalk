use axum::{routing::get, Router, response::Html};
use std::net::SocketAddr;

pub fn start_server() {
    // Use tokio runtime for async main
    tokio_main();
}

#[tokio::main]
async fn tokio_main() {
    // Build our application with a single route
    let app = Router::new().route("/", get(root));

    // Set the address to serve on
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Serving Binwalk Web UI at http://{}", addr);

    // Create a TcpListener as required by axum 0.7+
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html lang=\"en\">
        <head>
            <meta charset=\"UTF-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
            <title>Binwalk Web UI</title>
        </head>
        <body>
            <h1>Welcome to Binwalk Web UI</h1>
            <p>This is a placeholder for the web interface.</p>
        </body>
        </html>
    "#)
} 