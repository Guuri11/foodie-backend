use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::errors::ProductError;
use super::value_objects::{ProductLocation, ProductOutcome, ProductStatus};

#[derive(Debug, Clone)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub status: ProductStatus,
    pub location: Option<ProductLocation>,
    pub quantity: Option<String>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub estimated_expiry_date: Option<DateTime<Utc>>,
    pub outcome: Option<ProductOutcome>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewProductProps {
    pub name: String,
    pub status: ProductStatus,
    pub location: Option<ProductLocation>,
    pub quantity: Option<String>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub estimated_expiry_date: Option<DateTime<Utc>>,
    pub outcome: Option<ProductOutcome>,
}

impl Product {
    pub fn new(props: NewProductProps) -> Result<Self, ProductError> {
        if props.name.trim().is_empty() {
            return Err(ProductError::NameEmpty);
        }

        if props.outcome.is_some() && props.status != ProductStatus::Finished {
            return Err(ProductError::OutcomeRequiresFinishedStatus);
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            name: props.name,
            status: props.status,
            location: props.location,
            quantity: props.quantity,
            expiry_date: props.expiry_date,
            estimated_expiry_date: props.estimated_expiry_date,
            outcome: props.outcome,
            created_at: now,
            updated_at: now,
        })
    }

    /// Constructor for data already persisted in the repository (no validation).
    #[allow(clippy::too_many_arguments)]
    pub fn from_repository(
        id: Uuid,
        name: String,
        status: ProductStatus,
        location: Option<ProductLocation>,
        quantity: Option<String>,
        expiry_date: Option<DateTime<Utc>>,
        estimated_expiry_date: Option<DateTime<Utc>>,
        outcome: Option<ProductOutcome>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            status,
            location,
            quantity,
            expiry_date,
            estimated_expiry_date,
            outcome,
            created_at,
            updated_at,
        }
    }
}
