use crate::adapters::http::server::HttpState;
use crate::domain::transfer::SenderInfo;
use crate::domain::transfer::{SessionState, TransferSession};
use axum::extract::{Multipart, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
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
pub async fn upload(
    State(state): State<HttpState>,
    Path(uuid): Path<Uuid>,
    header: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let session_res = state.storage_service.get_session_by_id(&uuid).await;
        if let Ok(session) = session_res {
            match header.get("token") {
                Some(token) => {
                    let token_str = token.to_str().unwrap();
                    if token_str != session.token.as_str() {
                        return (
                            StatusCode::UNAUTHORIZED,
                            "No such id was accepted for user!",
                        );
                    }
                }
                None => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        "No such id was accepted for user!",
                    )
                }
            }
            let filename = field.file_name().unwrap_or("unknown.bin").to_string();

            println!("Receiving: {}", filename);

            let data = field.bytes().await.unwrap();

            if let Some(download_dir) = dirs::download_dir() {
                let file_path = download_dir.join(&filename);
                match tokio::fs::write(&file_path, &data).await {
                    Ok(_) => {
                        println!("Saved file with path {:?}", file_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to save file {} with error {}", &filename, &e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file!");
                    }
                }
            } else {
                println!(
                    "Failed to found download directory! Try to add download dir to environment"
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Download directory wasn't found",
                );
            }
        }
    }
    return (StatusCode::OK, "Success on uploading files!");
}

pub async fn handle_prepare(
    State(state): State<HttpState>,
    Json(sender): Json<SenderInfo>,
) -> Json<PrepareResponse> {
    let is_accepted = state.user_service.ask_accept_files(&sender);
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
