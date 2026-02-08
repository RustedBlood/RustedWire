use axum::{
    routing::{get, post},
    Router,
};

use crate::adapters::http::handlers::{index, upload};

pub async fn start_server() {
    let app = Router::new()
        .route("/", get(index))
        .route("/upload", post(upload));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("starting server...");
    axum::serve(listener, app).await.unwrap();
}
