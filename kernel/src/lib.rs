pub mod adapters;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {

    use uuid::Uuid;

    use crate::{adapters, application::ports::SessionStorageRepository, domain};
    #[tokio::test]
    async fn test_storage() {
        let storage = adapters::db::storage::SessionStorage::new();
        let first_session = domain::transfer::TransferSession {
            id: Uuid::new_v4(),
            token: "I am a token".to_string(),
            files: Vec::new(),
            sender: "CachyOs".to_string(),
            reciever: "Windows".to_string(),
            state: domain::transfer::SessionState::Proposed,
        };
        let second_session = domain::transfer::TransferSession {
            id: Uuid::new_v4(),
            token: "I am a token".to_string(),
            files: Vec::new(),
            sender: "CachyOs".to_string(),
            reciever: "Arch".to_string(),
            state: domain::transfer::SessionState::Proposed,
        };

        storage.save_session(&first_session).await.unwrap();
        storage.save_session(&second_session).await.unwrap();

        let session = storage.get_session_by_id(first_session.id).await.unwrap();
        println!("{}\n{}", session.id, session.token);
    }
}
