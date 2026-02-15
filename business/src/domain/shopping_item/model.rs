use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::errors::ShoppingItemError;

#[derive(Debug, Clone)]
pub struct ShoppingItem {
    pub id: Uuid,
    pub name: String,
    pub product_id: Option<Uuid>,
    pub is_bought: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ShoppingItem {
    pub fn new(name: String, product_id: Option<Uuid>) -> Result<Self, ShoppingItemError> {
        if name.trim().is_empty() {
            return Err(ShoppingItemError::NameEmpty);
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            product_id,
            is_bought: false,
            created_at: now,
            updated_at: now,
        })
    }

    /// Constructor for data already persisted in the repository (no validation).
    pub fn from_repository(
        id: Uuid,
        name: String,
        product_id: Option<Uuid>,
        is_bought: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            product_id,
            is_bought,
            created_at,
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_item_when_name_valid() {
        let result = ShoppingItem::new("Extra Virgin Olive Oil".to_string(), None);

        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.name, "Extra Virgin Olive Oil");
        assert!(item.product_id.is_none());
    }

    #[test]
    fn should_reject_when_name_empty() {
        let result = ShoppingItem::new("".to_string(), None);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShoppingItemError::NameEmpty));
    }

    #[test]
    fn should_reject_when_name_only_whitespace() {
        let result = ShoppingItem::new("   ".to_string(), None);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShoppingItemError::NameEmpty));
    }

    #[test]
    fn should_default_is_bought_to_false() {
        let item = ShoppingItem::new("Milk".to_string(), Some(Uuid::new_v4())).unwrap();

        assert!(!item.is_bought);
    }

    #[test]
    fn should_associate_product_id_when_provided() {
        let product_id = Uuid::new_v4();
        let item = ShoppingItem::new("Milk".to_string(), Some(product_id)).unwrap();

        assert_eq!(item.product_id, Some(product_id));
    }
}
