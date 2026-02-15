use async_trait::async_trait;

use crate::domain::product::errors::ProductError;
use crate::domain::product::services::ReceiptScanResult;

pub struct ScanReceiptParams {
    pub image_base64: String,
}

#[async_trait]
pub trait ScanReceiptUseCase: Send + Sync {
    async fn execute(&self, params: ScanReceiptParams) -> Result<ReceiptScanResult, ProductError>;
}
