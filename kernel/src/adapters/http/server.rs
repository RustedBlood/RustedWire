use std::sync::Arc;

use crate::adapters::db::storage;
use crate::adapters::http::handlers::{index, upload};
use crate::application::ports;
use axum::{
    routing::{get, post},
    Router,
};

#[derive(Clone)]
pub struct HttpState {
    pub storage_service: Arc<dyn ports::SessionStorageRepository>,
    pub user_service: Arc<dyn ports::UserInteractionService>,
}

pub async fn start_server(user_service: Arc<dyn ports::UserInteractionService>) {
    let state = HttpState {
        storage_service: Arc::new(storage::SessionStorage::new()),
        user_service: user_service,
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/upload", post(upload))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("starting server...");
    axum::serve(listener, app).await.unwrap();
}
