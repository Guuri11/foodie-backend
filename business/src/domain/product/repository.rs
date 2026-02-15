use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::errors::RepositoryError;

use super::model::Product;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Product>, RepositoryError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Product, RepositoryError>;
    async fn save(&self, product: &Product) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn get_active_products(&self) -> Result<Vec<Product>, RepositoryError>;
}
