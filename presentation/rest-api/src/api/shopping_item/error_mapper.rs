use poem::http::StatusCode;
use poem_openapi::payload::Json;

use business::domain::shopping_item::errors::ShoppingItemError;

use crate::api::error::{ErrorResponse, IntoErrorResponse};

impl IntoErrorResponse for ShoppingItemError {
    fn into_error_response(self) -> (StatusCode, Json<ErrorResponse>) {
        let (status, name, message) = match &self {
            ShoppingItemError::NameEmpty => (
                StatusCode::BAD_REQUEST,
                "ValidationError",
                "shopping_item.name_empty",
            ),
            ShoppingItemError::NotFound => {
                (StatusCode::NOT_FOUND, "NotFound", "shopping_item.not_found")
            }
            ShoppingItemError::AlreadyExists => (
                StatusCode::CONFLICT,
                "Conflict",
                "shopping_item.already_exists",
            ),
            ShoppingItemError::Repository(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                "repository.persistence",
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
