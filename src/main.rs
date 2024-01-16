use axum::{response, routing, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", routing::get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listen on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> response::Html<&'static str> {
    response::Html("<h1> hello world </h1>")
}
