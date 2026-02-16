use std::sync::Arc;

use crate::adapters::http::server::HttpState;
use crate::domain::transfer::SenderInfo;
use axum::extract::{Multipart, State};
use axum::response::IntoResponse;
use axum::Json;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
enum PrepareResponseStatus {
    Declined,
    Accepted,
}
use crate::domain::token::generate_transfer_token;
struct PrepareResponse {
    status: PrepareResponseStatus,
    uuid: Option<Uuid>,
    token: Option<String>,
}
pub async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().unwrap_or("unknown.bin").to_string();

        println!("Receiving: {}", filename);

        let data = field.bytes().await.unwrap();

        let mut file = tokio::fs::File::create(format!("~/uploads/{}", filename))
            .await
            .unwrap();

        file.write_all(&data).await.unwrap();
    }
}

pub async fn handle_prepare(
    State(state): State<Arc<HttpState>>,
    Json(sender): Json<SenderInfo>,
) -> Json<PrepareResponse> {
    let is_accepted = state.user_service.ask_accept_files(&sender).await;
    if is_accepted {
        let response = PrepareResponse {
            status: PrepareResponseStatus::Accepted,
            uuid: Some(Uuid::new_v4()),
            token: Some(generate_transfer_token().await),
        };
        return Json(response);
    } else {
        let response = PrepareResponse {
            status: PrepareResponseStatus::Declined,
            uuid: None,
            token: None,
        };
        return Json(response);
    }
}

pub async fn index() -> &'static str {
    "Ok"
}
