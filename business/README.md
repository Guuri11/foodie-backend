# Business Layer

The business layer is the core of the application. It contains all domain logic, entities, value objects, and use cases. This layer has **zero dependencies** on infrastructure or presentation — it defines contracts (ports) that outer layers implement.

## Structure

```
business/
  src/
    domain/
      <entity>/
        model.rs              # Domain model (aggregate root)
        value_objects.rs       # Type-safe wrappers with validation
        errors.rs              # Domain-specific error enum
        repository.rs          # Repository trait (port)
        queries.rs             # Query-specific structs (optional)
        use_cases/
          create.rs            # Params + Trait + Impl struct
          update.rs
          delete.rs
          get_by_id.rs
          ...
      common/
        email.rs               # Shared value objects
        phone.rs
        paginated_result.rs    # Generic pagination wrapper
      errors.rs                # Shared errors (RepositoryError)
      logger.rs                # Logger trait (port)
      events.rs                # Event types and publisher trait
    application/
      <entity>/
        create.rs              # Use case implementation
        update.rs
        delete.rs
        get_by_id.rs
        ...
    tests/
      mocks/                   # Centralized mock implementations
    lib.rs                     # Module declarations
```

## Domain Models

Domain models are aggregate roots that enforce business invariants. They use a **Props-based constructor pattern** with two entry points:

- `new(props)` — validates all fields, returns `Result<Self, EntityError>`. Used when creating new entities.
- `from_repository(...)` — bypasses validation, used only by the persistence layer to reconstruct entities from the database.

```rust
pub struct EntityProps {
    pub name: String,
    pub email: String,
    pub status: EntityStatus,
}

pub struct Entity {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub name: String,
    pub email: Email,       // Value object — validated
    pub status: EntityStatus,
}

impl Entity {
    /// Creates a new entity with full validation.
    pub fn new(props: EntityProps) -> Result<Self, EntityError> {
        if props.name.trim().is_empty() {
            return Err(EntityError::ValidationError("name_empty".to_string()));
        }
        let email = Email::new(props.email)
            .map_err(|_| EntityError::InvalidEmail)?;

        Ok(Self {
            id: Uuid::new_v4(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            name: props.name.trim().to_string(),
            email,
            status: props.status,
        })
    }

    /// Reconstructs from persistence. No validation — data is already trusted.
    pub fn from_repository(
        id: Uuid,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
        name: String,
        email: Email,
        status: EntityStatus,
    ) -> Self {
        Self { id, created_at, updated_at, name, email, status }
    }

    /// Business logic methods return Result when they can fail.
    pub fn activate(&mut self) -> Result<(), EntityError> {
        if self.status == EntityStatus::Active {
            return Err(EntityError::ValidationError("already_active".to_string()));
        }
        self.status = EntityStatus::Active;
        self.updated_at = Utc::now().naive_utc();
        Ok(())
    }
}
```

## Value Objects

Value objects are type-safe wrappers that validate data at construction time. They are immutable and compared by value.

### Simple wrapper with validation

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email {
    value: String,
}

impl Email {
    pub fn new(value: String) -> Result<Self, EmailError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(EmailError::Empty);
        }
        // Validate format...
        Ok(Self { value: trimmed.to_string() })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_string(self) -> String {
        self.value
    }
}
```

### Enum-based value objects

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityStatus {
    Active,
    Inactive,
    Suspended,
}

impl EntityStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "Active",
            Self::Inactive => "Inactive",
            Self::Suspended => "Suspended",
        }
    }
}

impl FromStr for EntityStatus {
    type Err = EntityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(Self::Active),
            "Inactive" => Ok(Self::Inactive),
            "Suspended" => Ok(Self::Suspended),
            _ => Err(EntityError::ValidationError("invalid_status".to_string())),
        }
    }
}
```

### Numeric value objects with bounds

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Latitude(pub f64);

impl Latitude {
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if !(-90.0..=90.0).contains(&value) {
            return Err(ValidationError::InvalidLatitude);
        }
        Ok(Self(value))
    }
}
```

## Domain Errors

Domain errors use `thiserror::Error` with **code-style identifiers** for internationalization. Error messages are machine-readable codes, not human-readable strings.

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EntityError {
    #[error("entity.validation_error.{0}")]
    ValidationError(String),

    #[error("entity.not_found")]
    NotFound,

    #[error("entity.duplicate")]
    Duplicate,

    #[error("entity.invalid_email")]
    InvalidEmail,

    #[error("entity.repository_error")]
    RepositoryError,

    #[error("entity.unknown")]
    Unknown,
}
```

### Error conversions

Use `From` trait to convert between error types, keeping use cases clean:

```rust
impl From<RepositoryError> for EntityError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::NotFound => EntityError::NotFound,
            RepositoryError::Duplicated => EntityError::Duplicate,
            _ => EntityError::RepositoryError,
        }
    }
}
```

### Shared repository error

A common `RepositoryError` enum is defined once and used across all repositories:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("repository.not_found")]
    NotFound,
    #[error("repository.persistence")]
    Persistence,
    #[error("repository.duplicated")]
    Duplicated,
    #[error("repository.database_error")]
    DatabaseError,
}
```

## Repository Traits (Ports)

Repository traits define the contract for data persistence. They live in the domain layer and are implemented by the infrastructure layer.

```rust
#[async_trait::async_trait]
pub trait EntityRepositoryTrait {
    async fn create(&self, entity: &Entity) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Entity>, RepositoryError>;
    async fn update(&self, entity: &Entity) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
    async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Entity>, RepositoryError>;
}
```

### Pagination support

Use the shared `PaginatedResult<T>` wrapper for paginated queries:

```rust
async fn find_by_criteria(
    &self,
    criteria: &SearchCriteria,
) -> Result<PaginatedResult<Entity>, RepositoryError>;
```

### Query-specific structs

For complex queries with filters, define criteria structs in `queries.rs`:

```rust
pub struct SearchCriteria {
    pub owner_id: Uuid,
    pub status: Option<EntityStatus>,
    pub date: Option<NaiveDate>,
    pub limit: i64,
    pub offset: i64,
}

/// Composite result for queries with JOINs
pub struct EntityWithDetails {
    pub entity: Entity,
    pub owner_name: String,
    pub category: Option<String>,
}
```

## Use Case Contracts

Each use case is defined in its own file inside `domain/<entity>/use_cases/`. A use case file contains three things:

1. **Params struct** — input data for the use case
2. **Trait** — the contract with an `execute` method
3. **Impl struct** — declares required dependencies

```rust
use std::sync::Arc;

/// Input parameters for the use case
#[derive(Debug, Clone)]
pub struct CreateEntityParams {
    pub name: String,
    pub email: String,
    pub owner_id: Uuid,
}

/// Use case contract
#[async_trait::async_trait]
pub trait CreateEntityUseCaseTrait {
    async fn execute(&self, params: CreateEntityParams) -> Result<Entity, EntityError>;
}

/// Implementation struct with injected dependencies
pub struct CreateEntityUseCaseImpl {
    pub repository: Arc<dyn EntityRepositoryTrait + Send + Sync>,
    pub logger: Arc<dyn Logger + Send + Sync>,
}
```

## Use Case Implementations

Implementations live in `application/<entity>/` with one file per use case. They implement the trait defined in the domain layer.

```rust
#[async_trait::async_trait]
impl CreateEntityUseCaseTrait for CreateEntityUseCaseImpl {
    async fn execute(&self, params: CreateEntityParams) -> Result<Entity, EntityError> {
        self.logger.info("Creating new entity");

        let entity = Entity::new(EntityProps {
            name: params.name,
            email: params.email,
            status: EntityStatus::Active,
        })?;

        self.repository.create(&entity).await?;

        self.logger.info(&format!("Entity created with id: {}", entity.id));
        Ok(entity)
    }
}
```

### Event publishing

Some use cases publish domain events after successful operations:

```rust
pub struct CompleteOperationUseCaseImpl {
    pub repository: Arc<dyn OperationRepositoryTrait + Send + Sync>,
    pub event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    pub logger: Arc<dyn Logger + Send + Sync>,
}

#[async_trait::async_trait]
impl CompleteOperationUseCaseTrait for CompleteOperationUseCaseImpl {
    async fn execute(&self, params: CompleteOperationParams) -> Result<Operation, OperationError> {
        // ... business logic ...

        self.repository.update(&operation).await?;

        // Fire-and-forget event publishing
        let _ = self.event_publisher.publish(BackofficeEvent::OperationCompleted {
            operation_id: operation.id,
        }).await;

        Ok(operation)
    }
}
```

## Testing

Tests live inside each use case implementation file in a dedicated `mod` with `#[cfg(test)]`.

### Naming convention

`should_<BUSINESS_EXPECTATION>_when_<BUSINESS_SCENARIO>`

### Structure (AAA pattern)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::mocks::*;

    #[tokio::test]
    async fn should_create_entity_when_valid_data_provided() {
        // Arrange — set up business scenario
        let mut mock_repo = MockEntityRepository::new();
        mock_repo.expect_create()
            .returning(|_| Ok(()));

        let use_case = CreateEntityUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: Arc::new(MockLogger::new()),
        };

        let params = CreateEntityParams {
            name: "Acme Corp".to_string(),
            email: "contact@acme.com".to_string(),
            owner_id: Uuid::new_v4(),
        };

        // Act — execute business operation
        let result = use_case.execute(params).await;

        // Assert — verify business outcome
        assert!(result.is_ok());
        let entity = result.unwrap();
        assert_eq!(entity.name, "Acme Corp");
    }

    #[tokio::test]
    async fn should_reject_entity_when_name_is_empty() {
        // Arrange
        let mock_repo = MockEntityRepository::new();
        let use_case = CreateEntityUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: Arc::new(MockLogger::new()),
        };

        let params = CreateEntityParams {
            name: "".to_string(),
            email: "contact@acme.com".to_string(),
            owner_id: Uuid::new_v4(),
        };

        // Act
        let result = use_case.execute(params).await;

        // Assert — verify error code, not message
        assert!(result.is_err());
        match result.unwrap_err() {
            EntityError::ValidationError(code) => assert_eq!(code, "name_empty"),
            _ => panic!("Expected ValidationError"),
        }
    }
}
```

### Mock helpers

Centralized mocks live in `tests/mocks/` and use `mockall`:

```rust
use mockall::mock;

mock! {
    pub EntityRepository {}

    #[async_trait::async_trait]
    impl EntityRepositoryTrait for EntityRepository {
        async fn create(&self, entity: &Entity) -> Result<(), RepositoryError>;
        async fn find_by_id(&self, id: &Uuid) -> Result<Option<Entity>, RepositoryError>;
        async fn update(&self, entity: &Entity) -> Result<(), RepositoryError>;
        // ...
    }
}
```

## Module Organization

The `lib.rs` file declares all domain and application modules:

```rust
// Domain modules (models, traits, errors)
pub mod domain {
    pub mod entity_a {
        pub mod model;
        pub mod value_objects;
        pub mod errors;
        pub mod repository;
        pub mod use_cases {
            pub mod create;
            pub mod update;
            pub mod get_by_id;
        }
    }
    pub mod entity_b { /* ... */ }
    pub mod common {
        pub mod email;
        pub mod paginated_result;
    }
    pub mod errors;
    pub mod logger;
    pub mod events;
}

// Application modules (use case implementations)
pub mod application {
    pub mod entity_a {
        pub mod create;
        pub mod update;
        pub mod get_by_id;
    }
    pub mod entity_b { /* ... */ }
}

// Test mocks
#[cfg(test)]
pub mod tests {
    pub mod mocks;
}
```

## Best Practices

- **No infrastructure dependencies.** The business layer must never import database drivers, HTTP clients, or framework types.
- **Validate at the boundary.** Domain models validate data in `new()`. Once constructed, the model is always in a valid state.
- **Use value objects for constrained types.** Emails, phone numbers, statuses, and coordinates should be value objects — not raw strings or floats.
- **One use case per file.** Each use case has a single responsibility and lives in its own file.
- **Always inject the logger.** Every use case implementation receives a `Logger` trait for structured logging.
- **Code-style error identifiers.** All error messages are machine-readable codes (e.g., `"name_empty"`, `"not_found"`) for i18n support.
- **Assert on error codes in tests.** Never assert on human-readable messages — always match error variants and check their code identifiers.
- **Use realistic test data.** Test data should reflect real business scenarios, not placeholder strings like `"test"` or `"a@b.c"`.
