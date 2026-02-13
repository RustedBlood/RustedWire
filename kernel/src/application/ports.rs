use crate::application::error;
use crate::domain::transfer::TransferSession;

pub trait SessionStorageRepository {
    fn new() -> Self;
    async fn save_session(
        &self,
        session: &TransferSession,
    ) -> Result<(), error::StorageServiceError>;
    async fn get_session_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<TransferSession, error::StorageServiceError>;
}
