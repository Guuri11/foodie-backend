use chrono::{DateTime, Utc};

/// Time range for recipe preparation.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeRange {
    /// ~10 minutes
    Quick,
    /// ~20 minutes
    Medium,
    /// ~30+ minutes
    Long,
}

impl std::fmt::Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeRange::Quick => write!(f, "quick"),
            TimeRange::Medium => write!(f, "medium"),
            TimeRange::Long => write!(f, "long"),
        }
    }
}

impl std::str::FromStr for TimeRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quick" => Ok(TimeRange::Quick),
            "medium" => Ok(TimeRange::Medium),
            "long" => Ok(TimeRange::Long),
            _ => Err(format!("Invalid time range: {}", s)),
        }
    }
}

/// Ingredient from user's pantry used in a suggestion.
#[derive(Debug, Clone)]
pub struct SuggestionIngredient {
    pub product_id: String,
    pub product_name: String,
    pub quantity: Option<String>,
    pub is_urgent: bool,
}

/// A cooking suggestion generated from available pantry products.
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub estimated_time: TimeRange,
    pub ingredients: Vec<SuggestionIngredient>,
    pub urgent_ingredients: Vec<String>,
    pub steps: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
}

/// Creates a new Suggestion with validation.
pub fn create_suggestion(
    id: String,
    title: String,
    description: Option<String>,
    estimated_time: TimeRange,
    ingredients: Vec<SuggestionIngredient>,
    steps: Option<Vec<String>>,
) -> Result<Suggestion, super::errors::SuggestionError> {
    if title.trim().is_empty() {
        return Err(super::errors::SuggestionError::InvalidSuggestion);
    }

    if ingredients.is_empty() {
        return Err(super::errors::SuggestionError::InvalidSuggestion);
    }

    let urgent_ingredients = ingredients
        .iter()
        .filter(|ing| ing.is_urgent)
        .map(|ing| ing.product_id.clone())
        .collect();

    Ok(Suggestion {
        id,
        title: title.trim().to_string(),
        description: description.map(|d| d.trim().to_string()),
        estimated_time,
        ingredients,
        urgent_ingredients,
        steps,
        created_at: Utc::now(),
    })
}
