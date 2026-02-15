use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use business::domain::shopping_item::model::ShoppingItem;

#[derive(Debug, FromRow)]
pub struct ShoppingItemEntity {
    pub id: Uuid,
    pub name: String,
    pub product_id: Option<Uuid>,
    pub is_bought: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ShoppingItemEntity {
    pub fn into_domain(self) -> ShoppingItem {
        ShoppingItem::from_repository(
            self.id,
            self.name,
            self.product_id,
            self.is_bought,
            self.created_at,
            self.updated_at,
        )
    }
}
