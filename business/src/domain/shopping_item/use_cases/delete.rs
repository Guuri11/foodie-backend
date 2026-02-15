use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::shopping_item::errors::ShoppingItemError;

pub struct DeleteShoppingItemParams {
    pub id: Uuid,
}

#[async_trait]
pub trait DeleteShoppingItemUseCase: Send + Sync {
    async fn execute(&self, params: DeleteShoppingItemParams) -> Result<(), ShoppingItemError>;
}
