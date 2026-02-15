#[derive(Debug, thiserror::Error)]
pub enum SuggestionError {
    #[error("suggestion.not_enough_products")]
    NotEnoughProducts,
    #[error("suggestion.generation_failed")]
    GenerationFailed,
    #[error("suggestion.invalid_suggestion")]
    InvalidSuggestion,
}
