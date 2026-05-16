use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone)]
pub struct TransferSession {
    pub id: Uuid,
    pub token: String,
    pub files: Vec<FileInfo>,
    pub sender: String,
    pub state: SessionState,
}

#[derive(Debug, Clone)]
pub enum SessionState {
    Proposed,     // Получен запрос, ждём подтверждения пользователя
    Confirmed,    // Пользователь разрешил передачу
    Transferring, // Идёт активная передача
    Completed,    // Передача успешно завершена
    Rejected,     // Пользователь отклонил запрос
    Cancelled,    // Инициатор отменил передачу
    Expired,      // Время ожидания истекло
}

#[derive(Deserialize, Serialize)]
pub struct SenderInfo {
    pub name: String,
    pub ip: String,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: i64,
    pub check_sum: Vec<u8>,
}
