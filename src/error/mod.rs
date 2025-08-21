use thiserror::Error;
use std::io;

pub type Result<T> = std::result::Result<T, TagItError>;

#[derive(Error, Debug)]
pub enum TagItError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Tag not found: {0}")]
    TagNotFound(String),

    #[error("Path already tagged with: {0}")]
    PathAlreadyTagged(String),

    #[error("Invalid tag name: {0}")]
    InvalidTagName(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl TagItError {
    pub fn user_friendly_message(&self) -> String {
        use colored::*;
        match self {
            Self::Database(e) => format!("{} Database error: {}", "Error:".red().bold(), e),
            Self::InvalidPath(p) => format!("{} Invalid path: {}", "Error:".red().bold(), p.blue()),
            Self::HomeDirNotFound => format!("{} Home directory not found", "Error:".red().bold()),
            Self::Io(e) => format!("{} IO error: {}", "Error:".red().bold(), e),
            Self::PathNotFound(p) => format!("{} Path not found: {}", "Error:".red().bold(), p.blue()),
            Self::TagNotFound(t) => format!("{} Tag not found: {}", "Error:".red().bold(), t.yellow()),
            Self::PathAlreadyTagged(t) => format!("{} Path already tagged with: {}", "Error:".red().bold(), t.yellow()),
            Self::InvalidTagName(t) => format!("{} Invalid tag name: {}", "Error:".red().bold(), t.yellow()),
            Self::Other(msg) => format!("{} {}", "Error:".red().bold(), msg),
        }
    }
} 