use axum::Router;
use tower_http::services;

#[tokio::main]
async fn main() {
    let serve_dir = services::ServeDir::new("assets")
        .not_found_service(services::ServeFile::new("assets/index.html"));
    let app = Router::new().fallback_service(serve_dir);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listen on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
