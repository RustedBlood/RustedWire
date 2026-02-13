use crate::application::error::StorageServiceError;
use crate::application::ports::SessionStorageRepository;
use crate::domain::transfer::TransferSession;
use dashmap::DashMap;

use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SessionStorage {
    pub storage: Arc<DashMap<Uuid, TransferSession>>,
}

impl SessionStorage {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(DashMap::new()),
        }
    }
}

impl SessionStorageRepository for SessionStorage {
    fn new() -> Self {
        Self::new()
    }
    async fn save_session(&self, session: &TransferSession) -> Result<(), StorageServiceError> {
        if self.storage.insert(session.id, session.clone()).is_some() {
            Err(StorageServiceError::FailedToAddSession)
        } else {
            Ok(())
        }
    }
    async fn get_session_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<TransferSession, StorageServiceError> {
        self.storage
            .get(&id)
            .map(|entry| entry.clone())
            .ok_or(StorageServiceError::SessionNotFound)
    }
}
