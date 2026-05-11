use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Ошибка чтения конфигурации: {0}")]
    Config(String),

    #[error("Ошибка парсинга JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Директория не найдена: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Файл не найден: {0}")]
    FileNotFound(PathBuf),

    #[error("Ошибка ввода/вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("Ошибка PDF: {0}")]
    Pdf(String),

    #[error("Ошибка печати: {0}")]
    Print(String),
}

pub type AppResult<T> = Result<T, AppError>;