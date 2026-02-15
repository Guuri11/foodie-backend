use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::errors::RepositoryError;

use super::model::ShoppingItem;

#[async_trait]
pub trait ShoppingItemRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<ShoppingItem>, RepositoryError>;
    async fn get_by_id(&self, id: Uuid) -> Result<ShoppingItem, RepositoryError>;
    async fn find_by_product_id(
        &self,
        product_id: Uuid,
    ) -> Result<Option<ShoppingItem>, RepositoryError>;
    async fn save(&self, item: &ShoppingItem) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn delete_by_product_id(&self, product_id: Uuid) -> Result<(), RepositoryError>;
    async fn delete_bought(&self) -> Result<u64, RepositoryError>;
}
