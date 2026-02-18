use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::shared::value_objects::UserId;
use crate::domain::shopping_item::errors::ShoppingItemError;

pub struct DeleteShoppingItemParams {
    pub id: Uuid,
    pub user_id: UserId,
}

#[async_trait]
pub trait DeleteShoppingItemUseCase: Send + Sync {
    async fn execute(&self, params: DeleteShoppingItemParams) -> Result<(), ShoppingItemError>;
}
