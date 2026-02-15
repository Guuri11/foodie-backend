use std::sync::Arc;

use poem_openapi::{OpenApi, param::Path, payload::Json};
use uuid::Uuid;

use business::domain::shopping_item::use_cases::clear_bought::ClearBoughtItemsUseCase;
use business::domain::shopping_item::use_cases::create::{
    CreateShoppingItemParams, CreateShoppingItemUseCase,
};
use business::domain::shopping_item::use_cases::delete::{
    DeleteShoppingItemParams, DeleteShoppingItemUseCase,
};
use business::domain::shopping_item::use_cases::get_all::GetAllShoppingItemsUseCase;
use business::domain::shopping_item::use_cases::update::{
    UpdateShoppingItemParams, UpdateShoppingItemUseCase,
};

use crate::api::error::{ErrorResponse, IntoErrorResponse};
use crate::api::shopping_item::dto::{
    ClearBoughtResponse, CreateShoppingItemRequest, ShoppingItemResponse, UpdateShoppingItemRequest,
};
use crate::api::tags::ApiTags;

pub struct ShoppingItemApi {
    create_use_case: Arc<dyn CreateShoppingItemUseCase>,
    get_all_use_case: Arc<dyn GetAllShoppingItemsUseCase>,
    update_use_case: Arc<dyn UpdateShoppingItemUseCase>,
    delete_use_case: Arc<dyn DeleteShoppingItemUseCase>,
    clear_bought_use_case: Arc<dyn ClearBoughtItemsUseCase>,
}

impl ShoppingItemApi {
    pub fn new(
        create_use_case: Arc<dyn CreateShoppingItemUseCase>,
        get_all_use_case: Arc<dyn GetAllShoppingItemsUseCase>,
        update_use_case: Arc<dyn UpdateShoppingItemUseCase>,
        delete_use_case: Arc<dyn DeleteShoppingItemUseCase>,
        clear_bought_use_case: Arc<dyn ClearBoughtItemsUseCase>,
    ) -> Self {
        Self {
            create_use_case,
            get_all_use_case,
            update_use_case,
            delete_use_case,
            clear_bought_use_case,
        }
    }
}

/// Shopping list management API
///
/// Endpoints for managing shopping list items.
#[OpenApi]
impl ShoppingItemApi {
    /// List all shopping items
    ///
    /// Returns all shopping list items ordered by creation date.
    #[oai(
        path = "/shopping-items",
        method = "get",
        tag = "ApiTags::ShoppingItems"
    )]
    async fn get_all(&self) -> GetAllShoppingItemsResponse {
        match self.get_all_use_case.execute().await {
            Ok(items) => {
                let responses: Vec<ShoppingItemResponse> =
                    items.into_iter().map(|i| i.into()).collect();
                GetAllShoppingItemsResponse::Ok(Json(responses))
            }
            Err(err) => {
                let (_status, json) = err.into_error_response();
                GetAllShoppingItemsResponse::InternalError(json)
            }
        }
    }

    /// Create a shopping item
    ///
    /// Adds a new item to the shopping list. If a product_id is provided and
    /// already exists in the list, returns the existing item.
    #[oai(
        path = "/shopping-items",
        method = "post",
        tag = "ApiTags::ShoppingItems"
    )]
    async fn create(&self, body: Json<CreateShoppingItemRequest>) -> CreateShoppingItemResponse {
        let product_id = match &body.0.product_id {
            Some(id) => match Uuid::parse_str(id) {
                Ok(uuid) => Some(uuid),
                Err(_) => {
                    return CreateShoppingItemResponse::BadRequest(Json(ErrorResponse {
                        name: "ValidationError".to_string(),
                        message: "shopping_item.invalid_product_id".to_string(),
                    }));
                }
            },
            None => None,
        };

        let params = CreateShoppingItemParams {
            name: body.0.name,
            product_id,
        };

        match self.create_use_case.execute(params).await {
            Ok(item) => CreateShoppingItemResponse::Created(Json(item.into())),
            Err(err) => {
                let (status, json) = err.into_error_response();
                match status.as_u16() {
                    400 => CreateShoppingItemResponse::BadRequest(json),
                    _ => CreateShoppingItemResponse::InternalError(json),
                }
            }
        }
    }

    /// Update a shopping item
    ///
    /// Updates the name and/or bought status of a shopping item.
    #[oai(
        path = "/shopping-items/:id",
        method = "put",
        tag = "ApiTags::ShoppingItems"
    )]
    async fn update(
        &self,
        id: Path<String>,
        body: Json<UpdateShoppingItemRequest>,
    ) -> UpdateShoppingItemResponse {
        let uuid = match Uuid::parse_str(&id.0) {
            Ok(uuid) => uuid,
            Err(_) => {
                return UpdateShoppingItemResponse::BadRequest(Json(ErrorResponse {
                    name: "ValidationError".to_string(),
                    message: "shopping_item.invalid_id".to_string(),
                }));
            }
        };

        let params = UpdateShoppingItemParams {
            id: uuid,
            name: body.0.name,
            is_bought: body.0.is_bought,
        };

        match self.update_use_case.execute(params).await {
            Ok(item) => UpdateShoppingItemResponse::Ok(Json(item.into())),
            Err(err) => {
                let (status, json) = err.into_error_response();
                match status.as_u16() {
                    400 => UpdateShoppingItemResponse::BadRequest(json),
                    404 => UpdateShoppingItemResponse::NotFound(json),
                    _ => UpdateShoppingItemResponse::InternalError(json),
                }
            }
        }
    }

    /// Delete a shopping item
    ///
    /// Permanently removes a shopping item from the list.
    #[oai(
        path = "/shopping-items/:id",
        method = "delete",
        tag = "ApiTags::ShoppingItems"
    )]
    async fn delete(&self, id: Path<String>) -> DeleteShoppingItemResponse {
        let uuid = match Uuid::parse_str(&id.0) {
            Ok(uuid) => uuid,
            Err(_) => {
                return DeleteShoppingItemResponse::BadRequest(Json(ErrorResponse {
                    name: "ValidationError".to_string(),
                    message: "shopping_item.invalid_id".to_string(),
                }));
            }
        };

        match self
            .delete_use_case
            .execute(DeleteShoppingItemParams { id: uuid })
            .await
        {
            Ok(()) => DeleteShoppingItemResponse::NoContent,
            Err(err) => {
                let (status, json) = err.into_error_response();
                match status.as_u16() {
                    404 => DeleteShoppingItemResponse::NotFound(json),
                    _ => DeleteShoppingItemResponse::InternalError(json),
                }
            }
        }
    }

    /// Clear bought items
    ///
    /// Removes all shopping items that have been marked as bought.
    #[oai(
        path = "/shopping-items/bought",
        method = "delete",
        tag = "ApiTags::ShoppingItems"
    )]
    async fn clear_bought(&self) -> ClearBoughtItemsResponse {
        match self.clear_bought_use_case.execute().await {
            Ok(count) => ClearBoughtItemsResponse::Ok(Json(ClearBoughtResponse { count })),
            Err(err) => {
                let (_status, json) = err.into_error_response();
                ClearBoughtItemsResponse::InternalError(json)
            }
        }
    }
}

#[derive(poem_openapi::ApiResponse)]
pub enum GetAllShoppingItemsResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<ShoppingItemResponse>>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum CreateShoppingItemResponse {
    #[oai(status = 201)]
    Created(Json<ShoppingItemResponse>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum UpdateShoppingItemResponse {
    #[oai(status = 200)]
    Ok(Json<ShoppingItemResponse>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum DeleteShoppingItemResponse {
    #[oai(status = 204)]
    NoContent,
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum ClearBoughtItemsResponse {
    #[oai(status = 200)]
    Ok(Json<ClearBoughtResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}
