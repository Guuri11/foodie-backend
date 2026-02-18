use async_trait::async_trait;

use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;
use crate::domain::shared::value_objects::UserId;

pub struct GetAllProductsParams {
    pub user_id: UserId,
}

#[async_trait]
pub trait GetAllProductsUseCase: Send + Sync {
    async fn execute(&self, params: GetAllProductsParams) -> Result<Vec<Product>, ProductError>;
}
