use async_trait::async_trait;

use crate::domain::product::model::Product;

use super::errors::SuggestionError;
use super::model::Suggestion;

/// Service port for generating cooking suggestions from available products.
#[async_trait]
pub trait SuggestionGeneratorService: Send + Sync {
    async fn generate(
        &self,
        products: &[Product],
        limit: usize,
    ) -> Result<Vec<Suggestion>, SuggestionError>;
}
