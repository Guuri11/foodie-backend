use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::product::errors::ProductError;
use crate::domain::shared::value_objects::UserId;

pub struct DeleteProductParams {
    pub id: Uuid,
    pub user_id: UserId,
}

#[async_trait]
pub trait DeleteProductUseCase: Send + Sync {
    async fn execute(&self, params: DeleteProductParams) -> Result<(), ProductError>;
}
