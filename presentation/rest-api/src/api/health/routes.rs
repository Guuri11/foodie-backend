use chrono::Utc;
use poem_openapi::{Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};

use crate::api::tags::ApiTags;

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct HealthCheckResponse {
    /// Service status
    pub status: String,
    /// Current server timestamp
    pub timestamp: String,
    /// Service version
    pub version: String,
}

/// Health API for monitoring and infrastructure checks
///
/// This module provides a health check endpoint for Kubernetes, Docker,
/// load balancers, and monitoring tools to verify the service is running.
pub struct Api;

impl Api {
    pub fn new() -> Self {
        Self
    }
}

#[OpenApi]
impl Api {
    /// Health check endpoint
    ///
    /// Returns the current status of the service.
    /// This endpoint is public and does not require authentication.
    ///
    /// ## Use Cases
    /// - Kubernetes liveness/readiness probes
    /// - Docker health checks
    /// - Load balancer health monitoring
    /// - Infrastructure monitoring tools
    ///
    /// ## Response
    /// - `status`: "healthy" if service is running
    /// - `timestamp`: Current server timestamp in ISO 8601 format
    /// - `version`: Service version from Cargo.toml
    #[oai(path = "/health", method = "get", tag = "ApiTags::Health")]
    async fn health_check(&self) -> Json<HealthCheckResponse> {
        Json(HealthCheckResponse {
            status: "healthy".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}
