use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::errors::ProductError;
use super::value_objects::ProductLocation;

/// Confidence level for AI-based estimations and identifications.
#[derive(Debug, Clone, PartialEq)]
pub enum Confidence {
    High,
    Medium,
    Low,
    None,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::High => write!(f, "high"),
            Confidence::Medium => write!(f, "medium"),
            Confidence::Low => write!(f, "low"),
            Confidence::None => write!(f, "none"),
        }
    }
}

impl std::str::FromStr for Confidence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(Confidence::High),
            "medium" => Ok(Confidence::Medium),
            "low" => Ok(Confidence::Low),
            "none" => Ok(Confidence::None),
            _ => Err(format!("Invalid confidence level: {}", s)),
        }
    }
}

/// Result of an expiry date estimation.
#[derive(Debug, Clone)]
pub struct ExpiryEstimation {
    pub date: Option<DateTime<Utc>>,
    pub confidence: Confidence,
}

/// Service port for estimating product expiry dates.
///
/// Considers product name, current status, and storage location
/// to estimate how long until the product expires.
#[async_trait]
pub trait ExpiryEstimatorService: Send + Sync {
    async fn estimate_expiry_date(
        &self,
        product_name: &str,
        status: &str,
        location: Option<String>,
    ) -> ExpiryEstimation;
}

/// Confidence level for product identification (high or low).
#[derive(Debug, Clone, PartialEq)]
pub enum IdentificationConfidence {
    High,
    Low,
}

impl std::fmt::Display for IdentificationConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentificationConfidence::High => write!(f, "high"),
            IdentificationConfidence::Low => write!(f, "low"),
        }
    }
}

impl std::str::FromStr for IdentificationConfidence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(IdentificationConfidence::High),
            "low" => Ok(IdentificationConfidence::Low),
            _ => Err(format!("Invalid identification confidence: {}", s)),
        }
    }
}

/// Method used to identify a product.
#[derive(Debug, Clone, PartialEq)]
pub enum IdentificationMethod {
    Barcode,
    Visual,
}

impl std::fmt::Display for IdentificationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentificationMethod::Barcode => write!(f, "barcode"),
            IdentificationMethod::Visual => write!(f, "visual"),
        }
    }
}

/// Result of a product identification.
#[derive(Debug, Clone)]
pub struct ProductIdentification {
    pub name: String,
    pub confidence: IdentificationConfidence,
    pub method: IdentificationMethod,
    pub suggested_location: Option<ProductLocation>,
    pub suggested_quantity: Option<String>,
}

/// Service port for identifying products by image or barcode.
#[async_trait]
pub trait ProductIdentifierService: Send + Sync {
    async fn identify_by_image(
        &self,
        image_base64: &str,
    ) -> Result<ProductIdentification, ProductError>;

    async fn identify_by_barcode(
        &self,
        barcode: &str,
    ) -> Result<ProductIdentification, ProductError>;
}

/// A single item extracted from a receipt.
#[derive(Debug, Clone)]
pub struct ReceiptItem {
    pub name: String,
    pub confidence: IdentificationConfidence,
}

/// Result of scanning a receipt image.
#[derive(Debug, Clone)]
pub struct ReceiptScanResult {
    pub items: Vec<ReceiptItem>,
}

/// Service port for extracting products from receipt images.
#[async_trait]
pub trait ReceiptScannerService: Send + Sync {
    async fn scan(&self, image_base64: &str) -> Result<ReceiptScanResult, ProductError>;
}
