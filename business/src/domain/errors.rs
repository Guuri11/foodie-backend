/// Repository errors for domain layer.
/// Use code-style identifiers for all error variants for i18n compatibility.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("repository.not_found")]
    NotFound,
    #[error("repository.persistence")]
    Persistence,
    #[error("repository.duplicated")]
    Duplicated,
    #[error("repository.database_error")]
    DatabaseError,
}

impl RepositoryError {
    pub fn not_found() -> Self {
        RepositoryError::NotFound
    }
    pub fn persistence() -> Self {
        RepositoryError::Persistence
    }
    pub fn duplicated() -> Self {
        RepositoryError::Duplicated
    }
    pub fn database_error() -> Self {
        RepositoryError::DatabaseError
    }
}
