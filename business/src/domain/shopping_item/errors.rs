#[derive(Debug, thiserror::Error)]
pub enum ShoppingItemError {
    #[error("shopping_item.name_empty")]
    NameEmpty,
    #[error("shopping_item.not_found")]
    NotFound,
    #[error("shopping_item.already_exists")]
    AlreadyExists,
    #[error("repository.persistence")]
    Repository(#[from] crate::domain::errors::RepositoryError),
}
