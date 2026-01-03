use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JournalError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse date: {0}")]
    DateParse(String),

    #[error("Template file not found at {0}")]
    _TemplateNotFound(PathBuf),

    #[error("Failed to parse SUMMARY.md: {0}")]
    _SummaryParse(String),

    #[error("Failed to open editor: {0}")]
    EditorFailed(String),

    #[error("Invalid configuration: {0}")]
    _InvalidConfig(String),

    #[error("Failed to fetch reminders: {0}")]
    RemindersFailed(String),
}

pub type Result<T> = std::result::Result<T, JournalError>;
