# Infrastructure Layer

The infrastructure layer contains adapters for external systems — databases, email services, authentication providers, and event broadcasting. It implements the ports (traits) defined in the business layer. **No business logic is permitted here.**

## Structure

```
infrastructure/
  persistence/
    src/
      <entity>/
        entity.rs            # Database entity (FromRow struct)
        repository.rs        # Repository trait implementation
      migrations/            # SQLx database migrations
      db.rs                  # Database connection and pool setup
      lib.rs                 # Module declarations
    Cargo.toml
  sse_broadcaster/           # Real-time SSE event broadcasting
  logger/                    # Tracing-based structured logging
  auth_service/              # Authentication provider adapter
  email/                     # Email adapters (dev/production)
```

## Persistent Entities

Database entities are flat structs that map directly to database rows using SQLx's `FromRow` derive. They are **not** domain models — they exist solely for serialization/deserialization.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityDb {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted: bool,
    pub deleted_at: Option<NaiveDateTime>,
    pub name: String,
    pub email: String,
    pub status: String,         // Stored as text, converted to enum
    pub quantity: BigDecimal,    // Precise decimal for financial/weight data
    pub owner_id: Uuid,
}
```

### Domain-to-database conversion

Two conversion traits handle the mapping between layers:

- **`TryFrom<EntityDb> for Entity`** — database to domain. Uses validation because database data may need enum parsing or value object construction.
- **`From<&Entity> for EntityDb`** — domain to database. No validation needed because domain models are always valid.

```rust
impl TryFrom<EntityDb> for Entity {
    type Error = EntityError;

    fn try_from(db: EntityDb) -> Result<Self, Self::Error> {
        let status = EntityStatus::from_str(&db.status)
            .map_err(|_| EntityError::ValidationError("invalid_status".to_string()))?;
        let email = Email::new(db.email)
            .map_err(|_| EntityError::InvalidEmail)?;

        Ok(Entity::from_repository(
            db.id,
            db.created_at,
            db.updated_at,
            db.name,
            email,
            status,
        ))
    }
}

impl From<&Entity> for EntityDb {
    fn from(entity: &Entity) -> Self {
        EntityDb {
            id: entity.id,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            deleted: false,
            deleted_at: None,
            name: entity.name.clone(),
            email: entity.email.as_str().to_string(),
            status: entity.status.as_str().to_string(),
            quantity: BigDecimal::from_str(&entity.quantity.to_string()).unwrap_or_default(),
            owner_id: entity.owner_id,
        }
    }
}
```

### PostgreSQL enum types

When the database uses PostgreSQL enums, define a matching Rust enum with SQLx derive:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "entity_status", rename_all = "PascalCase")]
pub enum EntityStatusDb {
    Active,
    Inactive,
    Suspended,
}
```

## Repository Implementations

Repositories implement the traits defined in the business layer. They receive a `PgPool` and an optional `Logger` via constructor injection.

```rust
pub struct EntityRepository {
    pub pool: PgPool,
    logger: Arc<dyn Logger>,
}

impl EntityRepository {
    pub fn new(pool: PgPool, logger: Arc<dyn Logger>) -> Self {
        Self { pool, logger }
    }
}

#[async_trait]
impl EntityRepositoryTrait for EntityRepository {
    async fn create(&self, entity: &Entity) -> Result<(), RepositoryError> {
        let db_entity = EntityDb::from(entity);

        sqlx::query(
            r#"INSERT INTO entity (id, created_at, updated_at, name, email, status, owner_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#
        )
        .bind(db_entity.id)
        .bind(db_entity.created_at)
        .bind(db_entity.updated_at)
        .bind(&db_entity.name)
        .bind(&db_entity.email)
        .bind(&db_entity.status)
        .bind(db_entity.owner_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().map(|c| c == "23505").unwrap_or(false) {
                    return RepositoryError::Duplicated;
                }
            }
            RepositoryError::DatabaseError
        })?;

        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Entity>, RepositoryError> {
        let result = sqlx::query_as::<_, EntityDb>(
            "SELECT * FROM entity WHERE id = $1 AND deleted = false"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?;

        match result {
            Some(db_entity) => {
                let domain = Entity::try_from(db_entity)
                    .map_err(|_| RepositoryError::Persistence)?;
                Ok(Some(domain))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, entity: &Entity) -> Result<(), RepositoryError> {
        let db_entity = EntityDb::from(entity);

        let rows = sqlx::query(
            r#"UPDATE entity
               SET name = $2, email = $3, status = $4, updated_at = $5
               WHERE id = $1 AND deleted = false"#
        )
        .bind(db_entity.id)
        .bind(&db_entity.name)
        .bind(&db_entity.email)
        .bind(&db_entity.status)
        .bind(db_entity.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?;

        if rows.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
```

### Queries with JOINs

For complex queries returning data from multiple tables, define inline `FromRow` structs:

```rust
#[derive(sqlx::FromRow)]
struct EntityWithOwnerRow {
    // Entity fields
    id: Uuid,
    name: String,
    status: String,
    // Owner fields (from LEFT JOIN)
    owner_name: Option<String>,
    owner_email: Option<String>,
}

async fn find_with_owner(&self, id: &Uuid) -> Result<Option<EntityWithOwner>, RepositoryError> {
    let row = sqlx::query_as::<_, EntityWithOwnerRow>(
        r#"SELECT e.id, e.name, e.status, o.name AS owner_name, o.email AS owner_email
           FROM entity e
           LEFT JOIN owner o ON e.owner_id = o.id
           WHERE e.id = $1 AND e.deleted = false"#
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|_| RepositoryError::DatabaseError)?;

    // Convert row to domain types...
}
```

### Error mapping

Map SQLx errors to domain `RepositoryError` variants:

| SQLx Error | PostgreSQL Code | RepositoryError |
|---|---|---|
| Unique violation | `23505` | `Duplicated` |
| Foreign key violation | `23503` | `DatabaseError` |
| Row not found | — | `NotFound` |
| Conversion failure | — | `Persistence` |
| Any other error | — | `DatabaseError` |

## Database Migrations

Migrations use SQLx and follow a timestamp-based naming convention.

### Naming convention

`YYYYMMDDHHMMSS_description.sql`

Example: `20251021142500_create_entity_table.sql`

### Table creation pattern

```sql
-- Create PostgreSQL enum type
CREATE TYPE entity_status AS ENUM ('Active', 'Inactive', 'Suspended');

-- Create table with standard metadata columns
CREATE TABLE entity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMP(3),

    -- Business fields
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    status entity_status NOT NULL DEFAULT 'Active',
    quantity DECIMAL(10, 2) NOT NULL,
    owner_id UUID NOT NULL REFERENCES owner(id) ON DELETE CASCADE,

    -- Optional fields
    description TEXT,
    metadata JSONB
);

-- Unique constraints
CREATE UNIQUE INDEX idx_unique_entity_email
    ON entity (email) WHERE deleted = FALSE;

-- Performance indexes
CREATE INDEX idx_entity_status
    ON entity (status) WHERE deleted = FALSE;

CREATE INDEX idx_entity_owner
    ON entity (owner_id) WHERE deleted = FALSE;

-- Case-insensitive search index
CREATE INDEX idx_entity_name_lower
    ON entity (LOWER(name)) WHERE deleted = FALSE;

-- Column documentation
COMMENT ON COLUMN entity.quantity IS 'Total quantity in metric tons';
COMMENT ON COLUMN entity.metadata IS 'Arbitrary JSON metadata from external systems';
```

### Key conventions

- **Soft deletes**: Every table has `deleted BOOLEAN DEFAULT FALSE` and `deleted_at TIMESTAMP(3)`.
- **Timestamp precision**: Use `TIMESTAMP(3)` for millisecond precision.
- **Partial indexes**: Add `WHERE deleted = FALSE` to indexes for performance.
- **UUID primary keys**: Use `gen_random_uuid()` for automatic ID generation.
- **Column comments**: Document non-obvious columns with `COMMENT ON COLUMN`.

## External Service Adapters

External service adapters implement business layer traits to integrate with third-party systems. They follow the adapter pattern from hexagonal architecture.

### Factory pattern for switchable implementations

When multiple implementations exist for the same trait (e.g., dev vs. production email), use a factory function:

```rust
pub enum EmailConfig {
    Production { api_key: String, from_email: String },
    Development { from_email: String, smtp_host: String, smtp_port: u16 },
}

pub fn create_email_sender(config: EmailConfig) -> Arc<dyn EmailSender + Send + Sync> {
    match config {
        EmailConfig::Production { api_key, from_email } => {
            Arc::new(ProductionEmailSender::new(api_key, from_email))
        }
        EmailConfig::Development { from_email, smtp_host, smtp_port } => {
            Arc::new(DevEmailSender::new(from_email, smtp_host, smtp_port))
        }
    }
}
```

### HTTP-based adapter

```rust
pub struct ExternalApiAdapter {
    client: reqwest::Client,
    base_url: String,
}

impl ExternalApiAdapter {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl ExternalServiceTrait for ExternalApiAdapter {
    async fn fetch_data(&self, id: &str) -> Result<DomainModel, ServiceError> {
        let response = self.client
            .get(&format!("{}/api/v1/data/{}", self.base_url, id))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|_| ServiceError::Unavailable)?;

        if !response.status().is_success() {
            return Err(ServiceError::NotFound);
        }

        let data: ExternalApiResponse = response.json().await
            .map_err(|_| ServiceError::InvalidResponse)?;

        DomainModel::try_from(data)
    }
}
```

### Authentication provider adapter

```rust
pub struct AuthProviderAdapter {
    domain: String,
    client_id: String,
    client_secret: String,
}

#[async_trait]
impl UserAuthService for AuthProviderAdapter {
    async fn get_user(&self, external_id: &str) -> Result<User, UserError> {
        let token = self.get_access_token().await?;
        let response = self.client
            .get(&format!("{}/api/v2/users/{}", self.domain, external_id))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| UserError::ExternalServiceUnavailable)?;

        let user_data: AuthProviderResponse = response.json().await
            .map_err(|_| UserError::ExternalServiceUnavailable)?;

        User::new(UserProps { /* map from response */ })
    }
}
```

## Event Publishing

Events are broadcast using tokio's broadcast channel with a fire-and-forget pattern:

```rust
pub struct EventBroadcaster {
    tx: broadcast::Sender<DomainEvent>,
    logger: Arc<dyn Logger>,
}

impl EventBroadcaster {
    pub fn new(capacity: usize, logger: Arc<dyn Logger>) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx, logger }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.tx.subscribe()
    }
}

#[async_trait]
impl EventPublisher for EventBroadcaster {
    async fn publish(&self, event: DomainEvent) -> Result<(), EventPublisherError> {
        match self.tx.send(event) {
            Ok(count) => {
                self.logger.debug(&format!("Event published to {} receivers", count));
                Ok(())
            }
            Err(_) => {
                // No subscribers — acceptable during startup or low-traffic periods
                Ok(())
            }
        }
    }
}
```

## Module Organization

### Persistence `lib.rs`

Each entity gets a module with `entity.rs` and `repository.rs`:

```rust
pub mod db;

pub mod entity_a {
    pub mod entity;
    pub mod repository;
}

pub mod entity_b {
    pub mod entity;
    pub mod repository;
}
```

### Cargo.toml dependency pattern

Infrastructure crates depend on the `business` crate via local path:

```toml
[dependencies]
business = { path = "../../business" }
async-trait = "0.1"
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "postgres",
    "uuid",
    "chrono",
    "macros",
    "derive",
    "migrate",
    "json",
    "bigdecimal",
] }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
thiserror = "2"
```

## Best Practices

- **No business logic.** Infrastructure adapters must not contain domain rules, validation, or business decisions. They only translate between external formats and domain types.
- **Implement business layer traits.** Every adapter implements a trait defined in `business/src/domain/`. The infrastructure layer never defines its own interfaces.
- **Map errors consistently.** Always convert SQLx/HTTP/external errors into domain-defined error types. Never expose infrastructure-specific errors to the business layer.
- **Use soft deletes.** All tables include `deleted` and `deleted_at` columns. Queries always filter with `WHERE deleted = FALSE`.
- **Inject PgPool, not connections.** Repositories receive the pool and let SQLx manage connection lifecycle.
- **Code-style error identifiers.** Even infrastructure error messages use code identifiers (e.g., `"repository.database_error"`) for consistency with the business layer.
- **One entity + repository per module.** Each domain entity maps to a persistence module containing exactly `entity.rs` and `repository.rs`.
- **Run migrations with SQLx CLI.** Always use `make sqlx/add-migration` for new migrations and `make sqlx/prepare` after modifying queries.
