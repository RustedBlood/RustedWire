use crate::application::error;
use crate::domain;
use crate::domain::transfer::{SessionState, TransferSession};
use async_trait::async_trait;

#[async_trait]
pub trait SessionStorageRepository: Send + Sync {
    async fn save_session(
        &self,
        session: &TransferSession,
    ) -> Result<(), error::StorageServiceError>;
    async fn get_session_by_id(
        &self,
        id: &uuid::Uuid,
    ) -> Result<TransferSession, error::StorageServiceError>;
    async fn update_session_state(
        &self,
        id: &uuid::Uuid,
        state: SessionState,
    ) -> Result<(), error::StorageServiceError>;
}

#[async_trait]
pub trait UserInteractionService: Send + Sync {
    fn ask_accept_files(&self, sender_info: &domain::transfer::SenderInfo) -> bool;
}
