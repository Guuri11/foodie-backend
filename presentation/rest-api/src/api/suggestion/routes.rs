use std::sync::Arc;

use poem_openapi::{OpenApi, param::Query, payload::Json};

use business::domain::shared::value_objects::UserId;
use business::domain::suggestion::use_cases::generate::{
    GenerateSuggestionsParams, GenerateSuggestionsUseCase,
};

use crate::api::error::{ErrorResponse, IntoErrorResponse};
use crate::api::security::FirebaseBearer;
use crate::api::suggestion::dto::SuggestionResponse;
use crate::api::tags::ApiTags;

pub struct SuggestionApi {
    generate_use_case: Arc<dyn GenerateSuggestionsUseCase>,
}

impl SuggestionApi {
    pub fn new(generate_use_case: Arc<dyn GenerateSuggestionsUseCase>) -> Self {
        Self { generate_use_case }
    }
}

/// Suggestion API
///
/// Endpoints for generating cooking suggestions based on available products.
#[OpenApi]
impl SuggestionApi {
    /// Generate cooking suggestions
    ///
    /// Returns AI-generated cooking suggestions based on available pantry products,
    /// prioritizing ingredients that are expiring soon.
    #[oai(path = "/suggestions", method = "get", tag = "ApiTags::Suggestions")]
    async fn get_suggestions(
        &self,
        auth: FirebaseBearer,
        /// Maximum number of suggestions to generate (default: 5)
        limit: Query<Option<usize>>,
    ) -> GetSuggestionsResponse {
        let user_id = UserId::new(auth.0);
        let limit = limit.0.unwrap_or(5).min(10);

        match self
            .generate_use_case
            .execute(GenerateSuggestionsParams { user_id, limit })
            .await
        {
            Ok(suggestions) => {
                let responses: Vec<SuggestionResponse> =
                    suggestions.into_iter().map(|s| s.into()).collect();
                GetSuggestionsResponse::Ok(Json(responses))
            }
            Err(err) => {
                let (_, json) = err.into_error_response();
                GetSuggestionsResponse::InternalError(json)
            }
        }
    }
}

#[derive(poem_openapi::ApiResponse)]
pub enum GetSuggestionsResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<SuggestionResponse>>),
    #[oai(status = 401)]
    Unauthorized(Json<ErrorResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}
