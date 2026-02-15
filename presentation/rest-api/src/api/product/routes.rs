use std::sync::Arc;

use poem_openapi::{OpenApi, param::Path, payload::Json};
use uuid::Uuid;

use business::domain::product::services::ExpiryEstimatorService;
use business::domain::product::use_cases::create::{CreateProductParams, CreateProductUseCase};
use business::domain::product::use_cases::delete::{DeleteProductParams, DeleteProductUseCase};
use business::domain::product::use_cases::estimate_expiry::{
    EstimateExpiryParams, EstimateExpiryUseCase,
};
use business::domain::product::use_cases::get_all::GetAllProductsUseCase;
use business::domain::product::use_cases::get_by_id::{
    GetProductByIdParams, GetProductByIdUseCase,
};
use business::domain::product::use_cases::identify::{
    IdentifyByBarcodeParams, IdentifyByImageParams, IdentifyProductUseCase,
};
use business::domain::product::use_cases::scan_receipt::{ScanReceiptParams, ScanReceiptUseCase};
use business::domain::product::use_cases::update::{UpdateProductParams, UpdateProductUseCase};

use crate::api::error::{ErrorResponse, IntoErrorResponse};
use crate::api::product::dto::{
    CreateProductRequest, EstimateExpiryDateRequest, ExpiryEstimationResponse,
    IdentifyByBarcodeRequest, IdentifyByImageRequest, ProductIdentificationResponse,
    ProductResponse, ReceiptScanResponse, ScanReceiptRequest, UpdateProductRequest,
};
use crate::api::tags::ApiTags;

pub struct ProductApi {
    create_use_case: Arc<dyn CreateProductUseCase>,
    get_all_use_case: Arc<dyn GetAllProductsUseCase>,
    get_by_id_use_case: Arc<dyn GetProductByIdUseCase>,
    update_use_case: Arc<dyn UpdateProductUseCase>,
    delete_use_case: Arc<dyn DeleteProductUseCase>,
    estimate_expiry_use_case: Arc<dyn EstimateExpiryUseCase>,
    expiry_estimator_service: Arc<dyn ExpiryEstimatorService>,
    identify_use_case: Arc<dyn IdentifyProductUseCase>,
    scan_receipt_use_case: Arc<dyn ScanReceiptUseCase>,
}

impl ProductApi {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        create_use_case: Arc<dyn CreateProductUseCase>,
        get_all_use_case: Arc<dyn GetAllProductsUseCase>,
        get_by_id_use_case: Arc<dyn GetProductByIdUseCase>,
        update_use_case: Arc<dyn UpdateProductUseCase>,
        delete_use_case: Arc<dyn DeleteProductUseCase>,
        estimate_expiry_use_case: Arc<dyn EstimateExpiryUseCase>,
        expiry_estimator_service: Arc<dyn ExpiryEstimatorService>,
        identify_use_case: Arc<dyn IdentifyProductUseCase>,
        scan_receipt_use_case: Arc<dyn ScanReceiptUseCase>,
    ) -> Self {
        Self {
            create_use_case,
            get_all_use_case,
            get_by_id_use_case,
            update_use_case,
            delete_use_case,
            estimate_expiry_use_case,
            expiry_estimator_service,
            identify_use_case,
            scan_receipt_use_case,
        }
    }
}

/// Product management API
///
/// Endpoints for creating, reading, updating, and deleting kitchen products.
#[OpenApi]
impl ProductApi {
    /// Create a new product
    ///
    /// Creates a new product in the kitchen inventory.
    #[oai(path = "/products", method = "post", tag = "ApiTags::Products")]
    async fn create_product(&self, body: Json<CreateProductRequest>) -> CreateProductResponse {
        let params = CreateProductParams {
            name: body.0.name,
            status: body.0.status.into(),
            location: body.0.location.map(|l| l.into()),
            quantity: body.0.quantity,
            expiry_date: body.0.expiry_date,
            estimated_expiry_date: body.0.estimated_expiry_date,
            outcome: body.0.outcome.map(|o| o.into()),
        };

        match self.create_use_case.execute(params).await {
            Ok(product) => CreateProductResponse::Created(Json(product.into())),
            Err(err) => {
                let (status, json) = err.into_error_response();
                match status.as_u16() {
                    400 => CreateProductResponse::BadRequest(json),
                    _ => CreateProductResponse::InternalError(json),
                }
            }
        }
    }

    /// List all active products
    ///
    /// Returns all products that are not in 'finished' status.
    #[oai(path = "/products", method = "get", tag = "ApiTags::Products")]
    async fn get_all_products(&self) -> GetAllProductsResponse {
        match self.get_all_use_case.execute().await {
            Ok(products) => {
                let responses: Vec<ProductResponse> =
                    products.into_iter().map(|p| p.into()).collect();
                GetAllProductsResponse::Ok(Json(responses))
            }
            Err(err) => {
                let (_status, json) = err.into_error_response();
                GetAllProductsResponse::InternalError(json)
            }
        }
    }

    /// Get a product by ID
    ///
    /// Returns a single product by its unique identifier.
    #[oai(path = "/products/:id", method = "get", tag = "ApiTags::Products")]
    async fn get_product_by_id(&self, id: Path<String>) -> GetProductByIdResponse {
        let uuid = match Uuid::parse_str(&id.0) {
            Ok(uuid) => uuid,
            Err(_) => {
                return GetProductByIdResponse::BadRequest(Json(ErrorResponse {
                    name: "ValidationError".to_string(),
                    message: "product.invalid_id".to_string(),
                }));
            }
        };

        match self
            .get_by_id_use_case
            .execute(GetProductByIdParams { id: uuid })
            .await
        {
            Ok(product) => GetProductByIdResponse::Ok(Json(product.into())),
            Err(err) => {
                let (status, json) = err.into_error_response();
                match status.as_u16() {
                    404 => GetProductByIdResponse::NotFound(json),
                    _ => GetProductByIdResponse::InternalError(json),
                }
            }
        }
    }

    /// Update a product
    ///
    /// Updates an existing product by its unique identifier.
    #[oai(path = "/products/:id", method = "put", tag = "ApiTags::Products")]
    async fn update_product(
        &self,
        id: Path<String>,
        body: Json<UpdateProductRequest>,
    ) -> UpdateProductResponse {
        let uuid = match Uuid::parse_str(&id.0) {
            Ok(uuid) => uuid,
            Err(_) => {
                return UpdateProductResponse::BadRequest(Json(ErrorResponse {
                    name: "ValidationError".to_string(),
                    message: "product.invalid_id".to_string(),
                }));
            }
        };

        let params = UpdateProductParams {
            id: uuid,
            name: body.0.name,
            status: body.0.status.into(),
            location: body.0.location.map(|l| l.into()),
            quantity: body.0.quantity,
            expiry_date: body.0.expiry_date,
            estimated_expiry_date: body.0.estimated_expiry_date,
            outcome: body.0.outcome.map(|o| o.into()),
        };

        match self.update_use_case.execute(params).await {
            Ok(product) => UpdateProductResponse::Ok(Json(product.into())),
            Err(err) => {
                let (status, json) = err.into_error_response();
                match status.as_u16() {
                    400 => UpdateProductResponse::BadRequest(json),
                    404 => UpdateProductResponse::NotFound(json),
                    _ => UpdateProductResponse::InternalError(json),
                }
            }
        }
    }

    /// Delete a product
    ///
    /// Permanently removes a product from the inventory.
    #[oai(path = "/products/:id", method = "delete", tag = "ApiTags::Products")]
    async fn delete_product(&self, id: Path<String>) -> DeleteProductResponse {
        let uuid = match Uuid::parse_str(&id.0) {
            Ok(uuid) => uuid,
            Err(_) => {
                return DeleteProductResponse::BadRequest(Json(ErrorResponse {
                    name: "ValidationError".to_string(),
                    message: "product.invalid_id".to_string(),
                }));
            }
        };

        match self
            .delete_use_case
            .execute(DeleteProductParams { id: uuid })
            .await
        {
            Ok(()) => DeleteProductResponse::NoContent,
            Err(err) => {
                let (status, json) = err.into_error_response();
                match status.as_u16() {
                    404 => DeleteProductResponse::NotFound(json),
                    _ => DeleteProductResponse::InternalError(json),
                }
            }
        }
    }

    /// Estimate expiry date for a product
    ///
    /// Uses AI to estimate when a product will expire based on its name,
    /// status, and storage location.
    #[oai(
        path = "/products/:id/estimate-expiry",
        method = "post",
        tag = "ApiTags::Products"
    )]
    async fn estimate_expiry(&self, id: Path<String>) -> EstimateExpiryResponse {
        let uuid = match Uuid::parse_str(&id.0) {
            Ok(uuid) => uuid,
            Err(_) => {
                return EstimateExpiryResponse::BadRequest(Json(ErrorResponse {
                    name: "ValidationError".to_string(),
                    message: "product.invalid_id".to_string(),
                }));
            }
        };

        match self
            .estimate_expiry_use_case
            .execute(EstimateExpiryParams { product_id: uuid })
            .await
        {
            Ok(product) => EstimateExpiryResponse::Ok(Json(product.into())),
            Err(err) => {
                let (status, json) = err.into_error_response();
                match status.as_u16() {
                    404 => EstimateExpiryResponse::NotFound(json),
                    _ => EstimateExpiryResponse::InternalError(json),
                }
            }
        }
    }

    /// Identify a product by image
    ///
    /// Uses AI vision to identify a food product from a photo.
    #[oai(
        path = "/products/identify/image",
        method = "post",
        tag = "ApiTags::Products"
    )]
    async fn identify_by_image(
        &self,
        body: Json<IdentifyByImageRequest>,
    ) -> IdentifyByImageResponse {
        match self
            .identify_use_case
            .execute_by_image(IdentifyByImageParams {
                image_base64: body.0.image_base64,
            })
            .await
        {
            Ok(identification) => IdentifyByImageResponse::Ok(Json(identification.into())),
            Err(err) => {
                let (_, json) = err.into_error_response();
                IdentifyByImageResponse::UnprocessableEntity(json)
            }
        }
    }

    /// Identify a product by barcode
    ///
    /// Looks up a product in the Open Food Facts database using its barcode.
    #[oai(
        path = "/products/identify/barcode",
        method = "post",
        tag = "ApiTags::Products"
    )]
    async fn identify_by_barcode(
        &self,
        body: Json<IdentifyByBarcodeRequest>,
    ) -> IdentifyByBarcodeResponse {
        match self
            .identify_use_case
            .execute_by_barcode(IdentifyByBarcodeParams {
                barcode: body.0.barcode,
            })
            .await
        {
            Ok(identification) => IdentifyByBarcodeResponse::Ok(Json(identification.into())),
            Err(err) => {
                let (_, json) = err.into_error_response();
                IdentifyByBarcodeResponse::UnprocessableEntity(json)
            }
        }
    }

    /// Scan a receipt image
    ///
    /// Uses AI to extract product names from a supermarket receipt photo.
    #[oai(
        path = "/products/scan-receipt",
        method = "post",
        tag = "ApiTags::Products"
    )]
    async fn scan_receipt(&self, body: Json<ScanReceiptRequest>) -> ScanReceiptResponse {
        match self
            .scan_receipt_use_case
            .execute(ScanReceiptParams {
                image_base64: body.0.image_base64,
            })
            .await
        {
            Ok(result) => ScanReceiptResponse::Ok(Json(result.into())),
            Err(err) => {
                let (_, json) = err.into_error_response();
                ScanReceiptResponse::UnprocessableEntity(json)
            }
        }
    }

    /// Estimate expiry date from product attributes
    ///
    /// Uses AI to estimate when a product will expire based on its name,
    /// status, and storage location. Does not require an existing product in the database.
    #[oai(
        path = "/products/estimate-expiry-date",
        method = "post",
        tag = "ApiTags::Products"
    )]
    async fn estimate_expiry_date(
        &self,
        body: Json<EstimateExpiryDateRequest>,
    ) -> EstimateExpiryDateResponse {
        let estimation = self
            .expiry_estimator_service
            .estimate_expiry_date(&body.0.product_name, &body.0.status, body.0.location)
            .await;

        EstimateExpiryDateResponse::Ok(Json(ExpiryEstimationResponse {
            date: estimation.date,
            confidence: estimation.confidence.into(),
        }))
    }
}

#[derive(poem_openapi::ApiResponse)]
pub enum CreateProductResponse {
    #[oai(status = 201)]
    Created(Json<ProductResponse>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum GetAllProductsResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<ProductResponse>>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum GetProductByIdResponse {
    #[oai(status = 200)]
    Ok(Json<ProductResponse>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum UpdateProductResponse {
    #[oai(status = 200)]
    Ok(Json<ProductResponse>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum DeleteProductResponse {
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
pub enum EstimateExpiryResponse {
    #[oai(status = 200)]
    Ok(Json<ProductResponse>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
    #[oai(status = 500)]
    InternalError(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum IdentifyByImageResponse {
    #[oai(status = 200)]
    Ok(Json<ProductIdentificationResponse>),
    #[oai(status = 422)]
    UnprocessableEntity(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum IdentifyByBarcodeResponse {
    #[oai(status = 200)]
    Ok(Json<ProductIdentificationResponse>),
    #[oai(status = 422)]
    UnprocessableEntity(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum ScanReceiptResponse {
    #[oai(status = 200)]
    Ok(Json<ReceiptScanResponse>),
    #[oai(status = 422)]
    UnprocessableEntity(Json<ErrorResponse>),
}

#[derive(poem_openapi::ApiResponse)]
pub enum EstimateExpiryDateResponse {
    #[oai(status = 200)]
    Ok(Json<ExpiryEstimationResponse>),
}
