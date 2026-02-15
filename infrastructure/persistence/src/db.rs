use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{path::Path, time::Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("database.connection_error")]
    ConnectionError,
    #[error("database.migration_error")]
    MigrationError,
}

/// Configuration for the database connection
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
}

impl DatabaseConfig {
    /// Creates a new database configuration with default values
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            max_connections: 5,
            acquire_timeout: Duration::from_secs(30),
        }
    }
}

/// Creates a PostgreSQL connection pool
pub async fn create_postgres_pool(config: &DatabaseConfig) -> Result<PgPool, DatabaseError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout)
        .connect(&config.connection_string)
        .await
        .map_err(|_| DatabaseError::ConnectionError)?;

    Ok(pool)
}

/// Runs database migrations from the specified directory
pub async fn run_migrations(pool: &PgPool, migrations_path: &str) -> Result<(), DatabaseError> {
    let path = Path::new(migrations_path);

    // Checks that the migrations directory exists
    if !path.exists() {
        return Err(DatabaseError::MigrationError);
    }

    // Runs the migrations
    sqlx::migrate::Migrator::new(path)
        .await
        .map_err(|_| DatabaseError::MigrationError)?
        .run(pool)
        .await
        .map_err(|_| DatabaseError::MigrationError)
}
