use poem::http::StatusCode;
use poem_openapi::payload::Json;

use business::domain::product::errors::ProductError;

use crate::api::error::{ErrorResponse, IntoErrorResponse};

impl IntoErrorResponse for ProductError {
    fn into_error_response(self) -> (StatusCode, Json<ErrorResponse>) {
        let (status, name, message) = match &self {
            ProductError::NameEmpty => (
                StatusCode::BAD_REQUEST,
                "ValidationError",
                "product.name_empty",
            ),
            ProductError::NotFound => (StatusCode::NOT_FOUND, "NotFound", "product.not_found"),
            ProductError::OutcomeRequiresFinishedStatus => (
                StatusCode::BAD_REQUEST,
                "ValidationError",
                "product.outcome_requires_finished_status",
            ),
            ProductError::IdentificationFailed => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "IdentificationError",
                "product.identification_failed",
            ),
            ProductError::ScanFailed => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "ScanError",
                "product.scan_failed",
            ),
            ProductError::Repository(_) => (
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
