use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;

pub struct EstimateExpiryParams {
    pub product_id: Uuid,
}

#[async_trait]
pub trait EstimateExpiryUseCase: Send + Sync {
    async fn execute(&self, params: EstimateExpiryParams) -> Result<Product, ProductError>;
}
