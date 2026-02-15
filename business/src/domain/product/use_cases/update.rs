use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;
use crate::domain::product::value_objects::{ProductLocation, ProductOutcome, ProductStatus};

pub struct UpdateProductParams {
    pub id: Uuid,
    pub name: String,
    pub status: ProductStatus,
    pub location: Option<ProductLocation>,
    pub quantity: Option<String>,
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    pub outcome: Option<ProductOutcome>,
}

#[async_trait]
pub trait UpdateProductUseCase: Send + Sync {
    async fn execute(&self, params: UpdateProductParams) -> Result<Product, ProductError>;
}
