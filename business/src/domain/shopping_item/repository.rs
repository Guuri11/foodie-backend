use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::errors::RepositoryError;
use crate::domain::shared::value_objects::UserId;

use super::model::ShoppingItem;

#[async_trait]
pub trait ShoppingItemRepository: Send + Sync {
    async fn get_all(&self, user_id: &UserId) -> Result<Vec<ShoppingItem>, RepositoryError>;
    async fn get_by_id(&self, id: Uuid, user_id: &UserId) -> Result<ShoppingItem, RepositoryError>;
    async fn find_by_product_id(
        &self,
        product_id: Uuid,
        user_id: &UserId,
    ) -> Result<Option<ShoppingItem>, RepositoryError>;
    async fn save(&self, item: &ShoppingItem) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Uuid, user_id: &UserId) -> Result<(), RepositoryError>;
    async fn delete_by_product_id(
        &self,
        product_id: Uuid,
        user_id: &UserId,
    ) -> Result<(), RepositoryError>;
    async fn delete_bought(&self, user_id: &UserId) -> Result<u64, RepositoryError>;
}
