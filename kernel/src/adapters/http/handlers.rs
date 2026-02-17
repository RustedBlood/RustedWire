use crate::adapters::http::server::HttpState;
use crate::domain::transfer::SenderInfo;
use crate::domain::transfer::{SessionState, TransferSession};
use axum::extract::{Multipart, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
enum PrepareResponseStatus {
    Declined,
    Accepted,
}
use crate::domain::token::generate_transfer_token;

#[derive(Deserialize, Serialize)]
pub struct PrepareResponse {
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
    State(state): State<HttpState>,
    Json(sender): Json<SenderInfo>,
) -> Json<PrepareResponse> {
    let is_accepted = state.user_service.ask_accept_files(&sender).await;
    if is_accepted {
        let uuid = Uuid::new_v4();
        let token = generate_transfer_token().await;
        let session_info = TransferSession {
            id: uuid,
            token: token.clone(),
            files: sender.files,
            sender: sender.name,
            state: SessionState::Confirmed,
        };
        match state.storage_service.save_session(&session_info).await {
            Ok(_) => println!("Session was saved to storage"),
            Err(e) => {
                println!("Failed to save new session with error: {e}");
                return Json(PrepareResponse {
                    status: PrepareResponseStatus::Declined,
                    uuid: None,
                    token: None,
                });
            }
        }
        let response = PrepareResponse {
            status: PrepareResponseStatus::Accepted,
            uuid: Some(uuid),
            token: Some(token),
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

pub async fn health() -> &'static str {
    "Ok"
}
