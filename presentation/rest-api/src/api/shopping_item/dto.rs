use chrono::{DateTime, Utc};
use poem_openapi::Object;

use business::domain::shopping_item::model::ShoppingItem;

#[derive(Debug, Clone, Object)]
pub struct CreateShoppingItemRequest {
    /// Item name (cannot be empty)
    pub name: String,
    /// Optional associated product ID
    #[oai(skip_serializing_if_is_none)]
    pub product_id: Option<String>,
}

#[derive(Debug, Clone, Object)]
pub struct UpdateShoppingItemRequest {
    /// New item name
    #[oai(skip_serializing_if_is_none)]
    pub name: Option<String>,
    /// Whether the item has been bought
    #[oai(skip_serializing_if_is_none)]
    pub is_bought: Option<bool>,
}

#[derive(Debug, Clone, Object)]
pub struct ShoppingItemResponse {
    /// Shopping item unique identifier
    pub id: String,
    /// Item name
    pub name: String,
    /// Associated product ID
    #[oai(skip_serializing_if_is_none)]
    pub product_id: Option<String>,
    /// Whether the item has been bought
    pub is_bought: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl From<ShoppingItem> for ShoppingItemResponse {
    fn from(item: ShoppingItem) -> Self {
        Self {
            id: item.id.to_string(),
            name: item.name,
            product_id: item.product_id.map(|id| id.to_string()),
            is_bought: item.is_bought,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(Debug, Clone, Object)]
pub struct ClearBoughtResponse {
    /// Number of items cleared
    pub count: u64,
}
