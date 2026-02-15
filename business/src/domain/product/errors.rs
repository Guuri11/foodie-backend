#[derive(Debug, thiserror::Error)]
pub enum ProductError {
    #[error("product.name_empty")]
    NameEmpty,
    #[error("product.not_found")]
    NotFound,
    #[error("product.outcome_requires_finished_status")]
    OutcomeRequiresFinishedStatus,
    #[error("product.identification_failed")]
    IdentificationFailed,
    #[error("product.scan_failed")]
    ScanFailed,
    #[error("repository.persistence")]
    Repository(#[from] crate::domain::errors::RepositoryError),
}
