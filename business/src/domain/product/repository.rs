use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::errors::RepositoryError;
use crate::domain::shared::value_objects::UserId;

use super::model::Product;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn get_all(&self, user_id: &UserId) -> Result<Vec<Product>, RepositoryError>;
    async fn get_by_id(&self, id: Uuid, user_id: &UserId) -> Result<Product, RepositoryError>;
    async fn save(&self, product: &Product) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid, user_id: &UserId) -> Result<(), RepositoryError>;
    async fn get_active_products(&self, user_id: &UserId) -> Result<Vec<Product>, RepositoryError>;
}
