use persistence::db::{DatabaseConfig, create_postgres_pool};
use sqlx::PgPool;
use std::env;

/// Initialize database connection pool from environment variables
///
/// Environment variables:
/// - DATABASE_URL: PostgreSQL connection string (required)
///
/// # Errors
/// Returns error if DATABASE_URL is not set or connection fails
pub async fn init_database() -> anyhow::Result<PgPool> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_postgres_pool(&DatabaseConfig::new(db_url)).await?;
    Ok(pool)
}
