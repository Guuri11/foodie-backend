use async_trait::async_trait;

use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::model::ShoppingItem;

#[async_trait]
pub trait GetAllShoppingItemsUseCase: Send + Sync {
    async fn execute(&self) -> Result<Vec<ShoppingItem>, ShoppingItemError>;
}
