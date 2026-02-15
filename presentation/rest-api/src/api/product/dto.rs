use chrono::{DateTime, Utc};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};

use business::domain::product::model::Product;
use business::domain::product::value_objects::{ProductLocation, ProductOutcome, ProductStatus};

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum ProductStatusDto {
    #[oai(rename = "new")]
    New,
    #[oai(rename = "opened")]
    Opened,
    #[oai(rename = "almost_empty")]
    AlmostEmpty,
    #[oai(rename = "finished")]
    Finished,
}

impl From<ProductStatus> for ProductStatusDto {
    fn from(status: ProductStatus) -> Self {
        match status {
            ProductStatus::New => ProductStatusDto::New,
            ProductStatus::Opened => ProductStatusDto::Opened,
            ProductStatus::AlmostEmpty => ProductStatusDto::AlmostEmpty,
            ProductStatus::Finished => ProductStatusDto::Finished,
        }
    }
}

impl From<ProductStatusDto> for ProductStatus {
    fn from(dto: ProductStatusDto) -> Self {
        match dto {
            ProductStatusDto::New => ProductStatus::New,
            ProductStatusDto::Opened => ProductStatus::Opened,
            ProductStatusDto::AlmostEmpty => ProductStatus::AlmostEmpty,
            ProductStatusDto::Finished => ProductStatus::Finished,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum ProductLocationDto {
    #[oai(rename = "fridge")]
    Fridge,
    #[oai(rename = "pantry")]
    Pantry,
    #[oai(rename = "freezer")]
    Freezer,
}

impl From<ProductLocation> for ProductLocationDto {
    fn from(loc: ProductLocation) -> Self {
        match loc {
            ProductLocation::Fridge => ProductLocationDto::Fridge,
            ProductLocation::Pantry => ProductLocationDto::Pantry,
            ProductLocation::Freezer => ProductLocationDto::Freezer,
        }
    }
}

impl From<ProductLocationDto> for ProductLocation {
    fn from(dto: ProductLocationDto) -> Self {
        match dto {
            ProductLocationDto::Fridge => ProductLocation::Fridge,
            ProductLocationDto::Pantry => ProductLocation::Pantry,
            ProductLocationDto::Freezer => ProductLocation::Freezer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum ProductOutcomeDto {
    #[oai(rename = "used")]
    Used,
    #[oai(rename = "thrown_away")]
    ThrownAway,
}

impl From<ProductOutcome> for ProductOutcomeDto {
    fn from(outcome: ProductOutcome) -> Self {
        match outcome {
            ProductOutcome::Used => ProductOutcomeDto::Used,
            ProductOutcome::ThrownAway => ProductOutcomeDto::ThrownAway,
        }
    }
}

impl From<ProductOutcomeDto> for ProductOutcome {
    fn from(dto: ProductOutcomeDto) -> Self {
        match dto {
            ProductOutcomeDto::Used => ProductOutcome::Used,
            ProductOutcomeDto::ThrownAway => ProductOutcome::ThrownAway,
        }
    }
}

#[derive(Debug, Clone, Object)]
pub struct CreateProductRequest {
    /// Product name (cannot be empty)
    pub name: String,
    /// Product status
    pub status: ProductStatusDto,
    /// Storage location
    #[oai(skip_serializing_if_is_none)]
    pub location: Option<ProductLocationDto>,
    /// Quantity description
    #[oai(skip_serializing_if_is_none)]
    pub quantity: Option<String>,
    /// Expiry date
    #[oai(skip_serializing_if_is_none)]
    pub expiry_date: Option<DateTime<Utc>>,
    /// Estimated expiry date
    #[oai(skip_serializing_if_is_none)]
    pub estimated_expiry_date: Option<DateTime<Utc>>,
    /// Product outcome (only valid when status is 'finished')
    #[oai(skip_serializing_if_is_none)]
    pub outcome: Option<ProductOutcomeDto>,
}

#[derive(Debug, Clone, Object)]
pub struct UpdateProductRequest {
    /// Product name (cannot be empty)
    pub name: String,
    /// Product status
    pub status: ProductStatusDto,
    /// Storage location
    #[oai(skip_serializing_if_is_none)]
    pub location: Option<ProductLocationDto>,
    /// Quantity description
    #[oai(skip_serializing_if_is_none)]
    pub quantity: Option<String>,
    /// Expiry date
    #[oai(skip_serializing_if_is_none)]
    pub expiry_date: Option<DateTime<Utc>>,
    /// Estimated expiry date
    #[oai(skip_serializing_if_is_none)]
    pub estimated_expiry_date: Option<DateTime<Utc>>,
    /// Product outcome (only valid when status is 'finished')
    #[oai(skip_serializing_if_is_none)]
    pub outcome: Option<ProductOutcomeDto>,
}

#[derive(Debug, Clone, Object)]
pub struct ProductResponse {
    /// Product unique identifier
    pub id: String,
    /// Product name
    pub name: String,
    /// Product status
    pub status: ProductStatusDto,
    /// Storage location
    #[oai(skip_serializing_if_is_none)]
    pub location: Option<ProductLocationDto>,
    /// Quantity description
    #[oai(skip_serializing_if_is_none)]
    pub quantity: Option<String>,
    /// Expiry date
    #[oai(skip_serializing_if_is_none)]
    pub expiry_date: Option<DateTime<Utc>>,
    /// Estimated expiry date
    #[oai(skip_serializing_if_is_none)]
    pub estimated_expiry_date: Option<DateTime<Utc>>,
    /// Product outcome
    #[oai(skip_serializing_if_is_none)]
    pub outcome: Option<ProductOutcomeDto>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            id: product.id.to_string(),
            name: product.name,
            status: product.status.into(),
            location: product.location.map(|l| l.into()),
            quantity: product.quantity,
            expiry_date: product.expiry_date,
            estimated_expiry_date: product.estimated_expiry_date,
            outcome: product.outcome.map(|o| o.into()),
            created_at: product.created_at,
            updated_at: product.updated_at,
        }
    }
}

// --- DTOs for expiry estimation ---

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum ConfidenceDto {
    #[oai(rename = "high")]
    High,
    #[oai(rename = "medium")]
    Medium,
    #[oai(rename = "low")]
    Low,
    #[oai(rename = "none")]
    None,
}

impl From<business::domain::product::services::Confidence> for ConfidenceDto {
    fn from(c: business::domain::product::services::Confidence) -> Self {
        match c {
            business::domain::product::services::Confidence::High => ConfidenceDto::High,
            business::domain::product::services::Confidence::Medium => ConfidenceDto::Medium,
            business::domain::product::services::Confidence::Low => ConfidenceDto::Low,
            business::domain::product::services::Confidence::None => ConfidenceDto::None,
        }
    }
}

/// Request to estimate expiry date based on product attributes.
#[derive(Debug, Clone, Object)]
pub struct EstimateExpiryDateRequest {
    /// Product name
    pub product_name: String,
    /// Product status (new, opened, almost_empty, finished)
    pub status: String,
    /// Storage location (fridge, pantry, freezer)
    #[oai(skip_serializing_if_is_none)]
    pub location: Option<String>,
}

/// Expiry date estimation result.
#[derive(Debug, Clone, Object)]
pub struct ExpiryEstimationResponse {
    /// Estimated expiry date (ISO 8601), or null if unable to estimate
    #[oai(skip_serializing_if_is_none)]
    pub date: Option<DateTime<Utc>>,
    /// Confidence level of the estimation
    pub confidence: ConfidenceDto,
}

// --- DTOs for product identification ---

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum IdentificationConfidenceDto {
    #[oai(rename = "high")]
    High,
    #[oai(rename = "low")]
    Low,
}

impl From<business::domain::product::services::IdentificationConfidence>
    for IdentificationConfidenceDto
{
    fn from(c: business::domain::product::services::IdentificationConfidence) -> Self {
        match c {
            business::domain::product::services::IdentificationConfidence::High => {
                IdentificationConfidenceDto::High
            }
            business::domain::product::services::IdentificationConfidence::Low => {
                IdentificationConfidenceDto::Low
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum IdentificationMethodDto {
    #[oai(rename = "barcode")]
    Barcode,
    #[oai(rename = "visual")]
    Visual,
}

impl From<business::domain::product::services::IdentificationMethod> for IdentificationMethodDto {
    fn from(m: business::domain::product::services::IdentificationMethod) -> Self {
        match m {
            business::domain::product::services::IdentificationMethod::Barcode => {
                IdentificationMethodDto::Barcode
            }
            business::domain::product::services::IdentificationMethod::Visual => {
                IdentificationMethodDto::Visual
            }
        }
    }
}

/// Request to identify a product by image.
#[derive(Debug, Clone, Object)]
pub struct IdentifyByImageRequest {
    /// Base64-encoded image data
    pub image_base64: String,
}

/// Request to identify a product by barcode.
#[derive(Debug, Clone, Object)]
pub struct IdentifyByBarcodeRequest {
    /// Barcode string (e.g., EAN-13)
    pub barcode: String,
}

/// Product identification result.
#[derive(Debug, Clone, Object)]
pub struct ProductIdentificationResponse {
    /// Identified product name
    pub name: String,
    /// Confidence level of the identification
    pub confidence: IdentificationConfidenceDto,
    /// Method used to identify the product
    pub method: IdentificationMethodDto,
    /// Suggested storage location
    #[oai(skip_serializing_if_is_none)]
    pub suggested_location: Option<ProductLocationDto>,
    /// Suggested quantity
    #[oai(skip_serializing_if_is_none)]
    pub suggested_quantity: Option<String>,
}

impl From<business::domain::product::services::ProductIdentification>
    for ProductIdentificationResponse
{
    fn from(id: business::domain::product::services::ProductIdentification) -> Self {
        Self {
            name: id.name,
            confidence: id.confidence.into(),
            method: id.method.into(),
            suggested_location: id.suggested_location.map(|l| l.into()),
            suggested_quantity: id.suggested_quantity,
        }
    }
}

/// Request to scan a receipt image.
#[derive(Debug, Clone, Object)]
pub struct ScanReceiptRequest {
    /// Base64-encoded receipt image data
    pub image_base64: String,
}

/// A single item extracted from a receipt.
#[derive(Debug, Clone, Object)]
pub struct ReceiptItemResponse {
    /// Product name
    pub name: String,
    /// Confidence level of the extraction
    pub confidence: IdentificationConfidenceDto,
}

/// Receipt scan result.
#[derive(Debug, Clone, Object)]
pub struct ReceiptScanResponse {
    /// Extracted product items
    pub items: Vec<ReceiptItemResponse>,
}

impl From<business::domain::product::services::ReceiptScanResult> for ReceiptScanResponse {
    fn from(result: business::domain::product::services::ReceiptScanResult) -> Self {
        Self {
            items: result
                .items
                .into_iter()
                .map(|item| ReceiptItemResponse {
                    name: item.name,
                    confidence: item.confidence.into(),
                })
                .collect(),
        }
    }
}
