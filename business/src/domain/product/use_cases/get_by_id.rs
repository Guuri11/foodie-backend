use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;
use crate::domain::shared::value_objects::UserId;

pub struct GetProductByIdParams {
    pub id: Uuid,
    pub user_id: UserId,
}

#[async_trait]
pub trait GetProductByIdUseCase: Send + Sync {
    async fn execute(&self, params: GetProductByIdParams) -> Result<Product, ProductError>;
}
