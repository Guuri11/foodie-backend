use poem::middleware::Cors;
use std::env;

/// Initialize CORS middleware for cross-origin requests
///
/// Environment variables:
/// - CORS_ALLOWED_ORIGINS: Comma-separated list of allowed origins
///   (default: "http://localhost:5173,http://localhost:1420,http://localhost:8080")
///
/// Configuration:
/// - Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS
/// - Headers: content-type, authorization, x-api-key
/// - Credentials: Enabled
///
pub fn init_cors() -> Cors {
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| {
        "http://localhost:5173,http://localhost:1420,http://localhost:8080".to_string()
    });

    let origins: Vec<&str> = allowed_origins.split(',').collect();

    Cors::new()
        .allow_origins(origins)
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
        .allow_headers(vec!["content-type", "authorization", "x-api-key"])
        .allow_credentials(true)
}
