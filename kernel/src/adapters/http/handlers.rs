use axum::extract::Multipart;
use axum::response::IntoResponse;
use tokio::io::AsyncWriteExt;
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

pub async fn handle_prepare() -> impl IntoResponse {}

pub async fn index() -> &'static str {
    "Ok"
}
