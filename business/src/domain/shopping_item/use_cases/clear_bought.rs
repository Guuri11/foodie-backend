use async_trait::async_trait;

use crate::domain::shared::value_objects::UserId;
use crate::domain::shopping_item::errors::ShoppingItemError;

pub struct ClearBoughtItemsParams {
    pub user_id: UserId,
}

#[async_trait]
pub trait ClearBoughtItemsUseCase: Send + Sync {
    async fn execute(&self, params: ClearBoughtItemsParams) -> Result<u64, ShoppingItemError>;
}
