use dotenvy::dotenv;

mod api;
mod config;
mod setup;

use config::{app_config::AppConfig, database_config};
use setup::{dependency_injection::DependencyContainer, server::Server};

/// REST API Entry Point
///
/// Initializes the application, wires dependencies, and starts the HTTP server.
///
/// Architecture follows ADR-002: Hexagonal Architecture with clear separation:
/// - config/: Application configuration (server, CORS, database, email)
/// - setup/: Dependency injection and server setup
/// - api/: Route handlers and DTOs (to be refactored to adapters/)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize tracing with RUST_LOG env filter
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // 2. Load environment variables
    dotenv().ok();

    // 3. Load configuration
    let config = AppConfig::from_env();

    // 4. Initialize database
    let pool = database_config::init_database().await?;

    // 5. Wire dependencies
    let container = DependencyContainer::new(pool).await?;

    // 6. Run server
    Server::run(config, container).await?;

    Ok(())
}
