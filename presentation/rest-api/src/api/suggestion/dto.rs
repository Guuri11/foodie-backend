use chrono::{DateTime, Utc};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};

use business::domain::suggestion::model::{Suggestion, TimeRange};

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum TimeRangeDto {
    #[oai(rename = "quick")]
    Quick,
    #[oai(rename = "medium")]
    Medium,
    #[oai(rename = "long")]
    Long,
}

impl From<TimeRange> for TimeRangeDto {
    fn from(t: TimeRange) -> Self {
        match t {
            TimeRange::Quick => TimeRangeDto::Quick,
            TimeRange::Medium => TimeRangeDto::Medium,
            TimeRange::Long => TimeRangeDto::Long,
        }
    }
}

#[derive(Debug, Clone, Object)]
pub struct SuggestionIngredientResponse {
    /// Product ID from user's pantry
    pub product_id: String,
    /// Product name
    pub product_name: String,
    /// Quantity description
    #[oai(skip_serializing_if_is_none)]
    pub quantity: Option<String>,
    /// Whether this ingredient is expiring soon
    pub is_urgent: bool,
}

#[derive(Debug, Clone, Object)]
pub struct SuggestionResponse {
    /// Suggestion unique identifier
    pub id: String,
    /// Recipe title
    pub title: String,
    /// Brief description
    #[oai(skip_serializing_if_is_none)]
    pub description: Option<String>,
    /// Estimated preparation time
    pub estimated_time: TimeRangeDto,
    /// Ingredients from user's pantry
    pub ingredients: Vec<SuggestionIngredientResponse>,
    /// Product IDs of urgent (expiring) ingredients
    pub urgent_ingredients: Vec<String>,
    /// Brief preparation steps
    #[oai(skip_serializing_if_is_none)]
    pub steps: Option<Vec<String>>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl From<Suggestion> for SuggestionResponse {
    fn from(s: Suggestion) -> Self {
        Self {
            id: s.id,
            title: s.title,
            description: s.description,
            estimated_time: s.estimated_time.into(),
            ingredients: s
                .ingredients
                .into_iter()
                .map(|i| SuggestionIngredientResponse {
                    product_id: i.product_id,
                    product_name: i.product_name,
                    quantity: i.quantity,
                    is_urgent: i.is_urgent,
                })
                .collect(),
            urgent_ingredients: s.urgent_ingredients,
            steps: s.steps,
            created_at: s.created_at,
        }
    }
}
