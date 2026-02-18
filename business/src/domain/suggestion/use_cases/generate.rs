use async_trait::async_trait;

use crate::domain::shared::value_objects::UserId;
use crate::domain::suggestion::errors::SuggestionError;
use crate::domain::suggestion::model::Suggestion;

pub struct GenerateSuggestionsParams {
    pub user_id: UserId,
    pub limit: usize,
}

#[async_trait]
pub trait GenerateSuggestionsUseCase: Send + Sync {
    async fn execute(
        &self,
        params: GenerateSuggestionsParams,
    ) -> Result<Vec<Suggestion>, SuggestionError>;
}
