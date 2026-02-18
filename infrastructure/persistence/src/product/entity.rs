use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use business::domain::product::model::Product;
use business::domain::product::value_objects::{ProductLocation, ProductOutcome, ProductStatus};
use business::domain::shared::value_objects::UserId;

#[derive(Debug, FromRow)]
pub struct ProductEntity {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub status: String,
    pub location: Option<String>,
    pub quantity: Option<String>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub estimated_expiry_date: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProductEntity {
    pub fn into_domain(self) -> Product {
        Product::from_repository(
            self.id,
            UserId::new(&self.user_id),
            self.name,
            self.status
                .parse::<ProductStatus>()
                .unwrap_or(ProductStatus::New),
            self.location
                .and_then(|l| l.parse::<ProductLocation>().ok()),
            self.quantity,
            self.expiry_date,
            self.estimated_expiry_date,
            self.outcome.and_then(|o| o.parse::<ProductOutcome>().ok()),
            self.created_at,
            self.updated_at,
        )
    }
}
