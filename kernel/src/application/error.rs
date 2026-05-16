use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageServiceError {
    #[error("Failed to add new session")]
    FailedToAddSession,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Failed to serialize session: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database connection failed: {0}")]
    Connection(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
