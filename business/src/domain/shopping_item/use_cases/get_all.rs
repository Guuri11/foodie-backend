use async_trait::async_trait;

use crate::domain::shared::value_objects::UserId;
use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::model::ShoppingItem;

pub struct GetAllShoppingItemsParams {
    pub user_id: UserId,
}

#[async_trait]
pub trait GetAllShoppingItemsUseCase: Send + Sync {
    async fn execute(
        &self,
        params: GetAllShoppingItemsParams,
    ) -> Result<Vec<ShoppingItem>, ShoppingItemError>;
}
