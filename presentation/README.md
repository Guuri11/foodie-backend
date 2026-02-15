# Presentation Layer

The presentation layer handles external requests and translates them into domain language. It contains REST API endpoints, CLI tools, and development servers. **No business logic is permitted here** — this layer is responsible for input validation, serialization, authentication, and error mapping.

## Structure

```
presentation/
  rest-api/
    src/
      api/
        <entity>/
          routes.rs          # Poem OpenAPI endpoint definitions
          dto.rs             # Request and response DTOs
          responses.rs       # ApiResponse enum with status variants
          error_mapper.rs    # Domain error to HTTP status mapping
        security.rs          # Authentication guards and helpers
        tags.rs              # OpenAPI documentation tags
        error.rs             # Shared ErrorResponse struct
      main.rs                # Server bootstrap and dependency injection
```

## Route Definitions

Routes use Poem OpenAPI macros on a struct that holds injected use cases as `Arc<dyn Trait>` fields.

```rust
pub struct EntityApi {
    create: Arc<dyn CreateEntityUseCaseTrait + Send + Sync>,
    get_by_id: Arc<dyn GetEntityByIdUseCaseTrait + Send + Sync>,
    list_all: Arc<dyn ListEntitiesUseCaseTrait + Send + Sync>,
}

impl EntityApi {
    pub fn new(
        create: Arc<dyn CreateEntityUseCaseTrait + Send + Sync>,
        get_by_id: Arc<dyn GetEntityByIdUseCaseTrait + Send + Sync>,
        list_all: Arc<dyn ListEntitiesUseCaseTrait + Send + Sync>,
    ) -> Self {
        Self { create, get_by_id, list_all }
    }
}

#[OpenApi]
impl EntityApi {
    /// Create a new entity
    ///
    /// Creates a new entity with the provided data.
    /// Requires authentication with a valid bearer token.
    #[oai(path = "/entities", method = "post", tag = "ApiTags::Entity")]
    pub async fn create_entity(
        &self,
        auth: Auth0Bearer,
        body: Json<CreateEntityRequest>,
    ) -> CreateEntityResponse {
        // 1. Validate authentication
        if let Err((status, error)) = validate_role(&auth).await {
            return CreateEntityResponse::from_status(status, error);
        }

        // 2. Map DTO to use case params
        let params = CreateEntityParams {
            name: body.0.name,
            email: body.0.email,
        };

        // 3. Execute use case and map result
        match self.create.execute(params).await {
            Ok(entity) => {
                let dto = EntityDto::from_domain(&entity);
                CreateEntityResponse::Created(Json(dto))
            }
            Err(err) => {
                let (status, error) = map_entity_error_to_response(err);
                CreateEntityResponse::from_status(status, error)
            }
        }
    }

    /// Get entity by ID
    #[oai(path = "/entities/:id", method = "get", tag = "ApiTags::Entity")]
    pub async fn get_by_id(
        &self,
        auth: Auth0Bearer,
        Path(id): Path<String>,
    ) -> GetEntityByIdResponse {
        let uuid = match Uuid::parse_str(&id) {
            Ok(uuid) => uuid,
            Err(_) => return GetEntityByIdResponse::BadRequest(Json(ErrorResponse {
                name: "entity.invalid_id".to_string(),
                message: "Invalid UUID format".to_string(),
            })),
        };

        match self.get_by_id.execute(GetEntityByIdParams { id: uuid }).await {
            Ok(entity) => {
                let dto = EntityDto::from_domain(&entity);
                GetEntityByIdResponse::Ok(Json(dto))
            }
            Err(err) => {
                let (status, error) = map_entity_error_to_response(err);
                GetEntityByIdResponse::from_status(status, error)
            }
        }
    }

    /// List entities with optional query filters
    #[oai(path = "/entities", method = "get", tag = "ApiTags::Entity")]
    pub async fn list_entities(
        &self,
        auth: Auth0Bearer,
        Query(limit): Query<Option<i64>>,
        Query(offset): Query<Option<i64>>,
        Query(status): Query<Option<String>>,
    ) -> ListEntitiesResponse {
        // ...
    }
}
```

## DTOs

DTOs (Data Transfer Objects) define the shape of API requests and responses. They use Poem OpenAPI's `Object` derive macro.

### Response DTOs

```rust
/// DTO for returning entity data in API responses
#[derive(Object, Debug, Clone, Serialize, Deserialize)]
#[oai(rename_all = "camelCase")]
pub struct EntityDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: String,
    pub created_at: String,
    pub owner_id: String,
}

impl EntityDto {
    /// Maps from domain model to DTO
    pub fn from_domain(entity: &Entity) -> Self {
        Self {
            id: entity.id.to_string(),
            name: entity.name.clone(),
            email: entity.email.as_str().to_string(),
            status: entity.status.as_str().to_string(),
            created_at: entity.created_at.to_string(),
            owner_id: entity.owner_id.to_string(),
        }
    }
}
```

### Request DTOs

```rust
/// Request DTO for creating an entity
#[derive(Object, Debug, Clone, Serialize, Deserialize)]
#[oai(rename_all = "camelCase")]
pub struct CreateEntityRequest {
    /// Entity name (required, min 2 characters)
    pub name: String,
    /// Contact email address
    pub email: String,
    /// Owner UUID
    pub owner_id: String,
}
```

### Enum DTOs

```rust
/// Enum DTO for entity status values
#[derive(Enum, Debug, Clone, Serialize, Deserialize)]
pub enum EntityStatusDto {
    Active,
    Inactive,
    Suspended,
}
```

### Paginated response DTOs

```rust
#[derive(Object, Debug, Clone, Serialize, Deserialize)]
#[oai(rename_all = "camelCase")]
pub struct PaginatedEntitiesResponse {
    pub items: Vec<EntityDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}
```

### Field renaming

Use `#[oai(rename_all = "camelCase")]` at the struct level for consistent JSON field naming. For individual fields that need custom names, use `#[oai(rename = "fieldName")]`:

```rust
#[derive(Object)]
pub struct EntityDto {
    pub id: Uuid,
    #[oai(rename = "logisticsCompanyId")]
    pub logistics_company_id: Uuid,
    #[oai(rename = "totemGroupId")]
    pub totem_group_id: Uuid,
}
```

## Response Types

Each endpoint defines an `ApiResponse` enum with variants for each possible HTTP status:

```rust
#[derive(ApiResponse)]
pub enum CreateEntityResponse {
    /// Entity created successfully
    #[oai(status = 201)]
    Created(Json<EntityDto>),

    /// Validation error
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),

    /// Authentication required
    #[oai(status = 401)]
    Unauthorized(Json<ErrorResponse>),

    /// Insufficient permissions
    #[oai(status = 403)]
    Forbidden(Json<ErrorResponse>),

    /// Duplicate entity
    #[oai(status = 409)]
    Conflict(Json<ErrorResponse>),

    /// Internal server error
    #[oai(status = 500)]
    InternalServerError(Json<ErrorResponse>),
}
```

### `from_status` helper

A helper method maps HTTP status codes to response variants, simplifying error handling:

```rust
impl CreateEntityResponse {
    pub fn from_status(status: StatusCode, error: ErrorResponse) -> Self {
        match status {
            StatusCode::BAD_REQUEST => Self::BadRequest(Json(error)),
            StatusCode::UNAUTHORIZED => Self::Unauthorized(Json(error)),
            StatusCode::FORBIDDEN => Self::Forbidden(Json(error)),
            StatusCode::CONFLICT => Self::Conflict(Json(error)),
            _ => Self::InternalServerError(Json(error)),
        }
    }
}
```

## Error Mapping

Each entity module has an `error_mapper.rs` that converts domain errors to HTTP responses. The mapping function returns a tuple of `(StatusCode, ErrorResponse)`.

```rust
pub fn map_entity_error_to_response(error: EntityError) -> (StatusCode, ErrorResponse) {
    match error {
        EntityError::ValidationError(ref code) => (
            StatusCode::BAD_REQUEST,
            ErrorResponse {
                name: format!("entity.validation_error.{}", code),
                message: error.to_string(),
            },
        ),
        EntityError::NotFound => (
            StatusCode::NOT_FOUND,
            ErrorResponse {
                name: "entity.not_found".to_string(),
                message: error.to_string(),
            },
        ),
        EntityError::Duplicate => (
            StatusCode::CONFLICT,
            ErrorResponse {
                name: "entity.duplicate".to_string(),
                message: error.to_string(),
            },
        ),
        EntityError::RepositoryError | EntityError::Unknown => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse {
                name: "entity.internal_error".to_string(),
                message: error.to_string(),
            },
        ),
    }
}
```

### ErrorResponse struct

A shared struct used across all error responses:

```rust
#[derive(Object, Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Machine-readable error code for i18n (e.g., "entity.not_found")
    pub name: String,
    /// Human-readable error description (for debugging only)
    pub message: String,
}
```

## Dependency Injection

All dependencies are wired in `main.rs` using a manual dependency injection pattern:

```rust
#[tokio::main]
async fn main() {
    // 1. Load configuration
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL required");

    // 2. Create infrastructure dependencies
    let pool = PgPool::connect(&database_url).await.expect("DB connection failed");
    let logger: Arc<dyn Logger> = Arc::new(TracingLogger::new());
    let email_sender = create_email_sender(email_config);
    let event_publisher: Arc<dyn EventPublisher> = Arc::new(EventBroadcaster::new(100, logger.clone()));

    // 3. Create repositories
    let entity_repo: Arc<dyn EntityRepositoryTrait + Send + Sync> =
        Arc::new(EntityRepository::new(pool.clone(), logger.clone()));

    // 4. Create use cases (inject repositories + logger)
    let create_entity: Arc<dyn CreateEntityUseCaseTrait + Send + Sync> =
        Arc::new(CreateEntityUseCaseImpl {
            repository: entity_repo.clone(),
            logger: logger.clone(),
        });

    let get_entity: Arc<dyn GetEntityByIdUseCaseTrait + Send + Sync> =
        Arc::new(GetEntityByIdUseCaseImpl {
            repository: entity_repo.clone(),
            logger: logger.clone(),
        });

    // 5. Create API structs (inject use cases)
    let entity_api = EntityApi::new(create_entity, get_entity, list_entities);

    // 6. Compose OpenAPI service
    let api_service = OpenApiService::new(
        (entity_api, other_api, webhook_api),
        "API Title",
        "1.0.0",
    )
    .server("http://localhost:8080");

    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();

    // 7. Configure middleware and start server
    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .at("/openapi.json", spec)
        .with(Cors::new())
        .with(Tracing);

    poem::Server::new(TcpListener::bind("0.0.0.0:8080"))
        .run(app)
        .await
        .expect("Server failed");
}
```

## Server Setup

### OpenAPI service composition

Multiple API structs are composed into a single OpenAPI service using a tuple:

```rust
let api_service = OpenApiService::new(
    (entity_api, owner_api, webhook_api, admin_api),
    "My API",
    "1.0.0",
)
.server("http://localhost:8080");
```

### Middleware

```rust
let app = Route::new()
    .nest("/", api_service)
    .nest("/docs", api_service.swagger_ui())
    .at("/openapi.json", api_service.spec_endpoint())
    .with(Cors::new().allow_origins_fn(|_| true))
    .with(Tracing);
```

### API Tags

Tags group endpoints in the Swagger UI:

```rust
#[derive(Tags)]
pub enum ApiTags {
    /// Entity management operations
    Entity,
    /// Owner management operations
    Owner,
    /// Webhook endpoints for external integrations
    Webhook,
    /// Administrative operations
    Admin,
}
```

## Module Organization

Each entity gets its own module with four files:

```
api/
  entity/
    mod.rs              # Module declarations
    routes.rs           # Endpoint definitions (#[OpenApi] impl)
    dto.rs              # Request and response DTOs
    responses.rs        # ApiResponse enums
    error_mapper.rs     # Domain error to HTTP mapping
  security.rs           # Auth guards and helpers
  tags.rs               # ApiTags enum
  error.rs              # Shared ErrorResponse
```

### Module declarations

```rust
// api/entity/mod.rs
pub mod dto;
pub mod error_mapper;
pub mod responses;
pub mod routes;
```

## Best Practices

- **No business logic.** Routes must not contain domain rules or decisions. They only validate input, call use cases, and map results to HTTP responses.
- **Validate input in the presentation layer.** Parse UUIDs, check required fields, and validate formats before calling use cases.
- **Use code-style error identifiers.** All error responses include a `name` field with a machine-readable code (e.g., `"entity.not_found"`) for frontend i18n.
- **camelCase for JSON fields.** Use `#[oai(rename_all = "camelCase")]` on DTOs for consistent API naming.
- **Map domain models to DTOs.** Never expose domain models directly in API responses. Always create a DTO with a `from_domain()` method.
- **One error mapper per entity.** Keep error mapping isolated per entity for maintainability.
- **Use `from_status` helpers.** Every `ApiResponse` enum should have a `from_status` helper for clean error propagation.
- **Authentication on every endpoint.** All endpoints receive an auth guard parameter (`Auth0Bearer` or `ApiKeyGuard`). Validate roles before executing business logic.
- **Update OpenAPI spec first.** Never implement or change an endpoint without first updating the route definition and DTOs that generate the OpenAPI contract.
