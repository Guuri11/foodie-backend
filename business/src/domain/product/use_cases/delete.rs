use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::product::errors::ProductError;

pub struct DeleteProductParams {
    pub id: Uuid,
}

#[async_trait]
pub trait DeleteProductUseCase: Send + Sync {
    async fn execute(&self, params: DeleteProductParams) -> Result<(), ProductError>;
}
