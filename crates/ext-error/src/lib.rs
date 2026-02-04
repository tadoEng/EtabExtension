use thiserror::Error;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Error, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("ETABS error: {0}")]
    Etabs(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("File system error: {0}")]
    FileSystem(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_typescript_bindings() {
        AppError::export(&Default::default()).expect("Failed to export AppError");
    }
}