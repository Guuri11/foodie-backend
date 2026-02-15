use async_trait::async_trait;

use crate::domain::product::errors::ProductError;
use crate::domain::product::services::ProductIdentification;

pub struct IdentifyByImageParams {
    pub image_base64: String,
}

pub struct IdentifyByBarcodeParams {
    pub barcode: String,
}

#[async_trait]
pub trait IdentifyProductUseCase: Send + Sync {
    async fn execute_by_image(
        &self,
        params: IdentifyByImageParams,
    ) -> Result<ProductIdentification, ProductError>;

    async fn execute_by_barcode(
        &self,
        params: IdentifyByBarcodeParams,
    ) -> Result<ProductIdentification, ProductError>;
}
