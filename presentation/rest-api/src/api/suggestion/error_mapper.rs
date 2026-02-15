use poem::http::StatusCode;
use poem_openapi::payload::Json;

use business::domain::suggestion::errors::SuggestionError;

use crate::api::error::{ErrorResponse, IntoErrorResponse};

impl IntoErrorResponse for SuggestionError {
    fn into_error_response(self) -> (StatusCode, Json<ErrorResponse>) {
        let (status, name, message) = match &self {
            SuggestionError::NotEnoughProducts => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "ValidationError",
                "suggestion.not_enough_products",
            ),
            SuggestionError::GenerationFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "GenerationError",
                "suggestion.generation_failed",
            ),
            SuggestionError::InvalidSuggestion => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "GenerationError",
                "suggestion.invalid_suggestion",
            ),
        };

        (
            status,
            Json(ErrorResponse {
                name: name.to_string(),
                message: message.to_string(),
            }),
        )
    }
}
