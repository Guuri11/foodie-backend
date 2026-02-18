use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;
use crate::domain::shared::value_objects::UserId;

pub struct EstimateExpiryParams {
    pub product_id: Uuid,
    pub user_id: UserId,
}

#[async_trait]
pub trait EstimateExpiryUseCase: Send + Sync {
    async fn execute(&self, params: EstimateExpiryParams) -> Result<Product, ProductError>;
}
