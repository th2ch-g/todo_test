pub mod label;
pub mod todo;

#[derive(Debug, thiserror::Error)]
enum RepositoryError {
    #[error("Unexpected Error: {0}")]
    Unexpected(String),
    #[error("NotFound, {0}")]
    NotFound(i32),
    #[error("Duplicate data {0}")]
    Duplicate(i32),
}
