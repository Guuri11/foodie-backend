use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::model::ShoppingItem;

pub struct CreateShoppingItemParams {
    pub name: String,
    pub product_id: Option<Uuid>,
}

#[async_trait]
pub trait CreateShoppingItemUseCase: Send + Sync {
    async fn execute(
        &self,
        params: CreateShoppingItemParams,
    ) -> Result<ShoppingItem, ShoppingItemError>;
}
