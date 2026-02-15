use async_trait::async_trait;

use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;

#[async_trait]
pub trait GetAllProductsUseCase: Send + Sync {
    async fn execute(&self) -> Result<Vec<Product>, ProductError>;
}
