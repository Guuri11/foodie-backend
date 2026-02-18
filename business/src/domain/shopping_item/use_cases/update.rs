use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::shared::value_objects::UserId;
use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::model::ShoppingItem;

pub struct UpdateShoppingItemParams {
    pub id: Uuid,
    pub user_id: UserId,
    pub name: Option<String>,
    pub is_bought: Option<bool>,
}

#[async_trait]
pub trait UpdateShoppingItemUseCase: Send + Sync {
    async fn execute(
        &self,
        params: UpdateShoppingItemParams,
    ) -> Result<ShoppingItem, ShoppingItemError>;
}
