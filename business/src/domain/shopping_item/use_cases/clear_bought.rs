use async_trait::async_trait;

use crate::domain::shopping_item::errors::ShoppingItemError;

#[async_trait]
pub trait ClearBoughtItemsUseCase: Send + Sync {
    async fn execute(&self) -> Result<u64, ShoppingItemError>;
}
