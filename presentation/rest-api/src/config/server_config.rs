use std::env;

/// Server configuration for HTTP listener
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub ip: String,
    pub port: String,
}

impl ServerConfig {
    /// Load server configuration from environment variables
    ///
    /// Environment variables:
    /// - SERVICE_IP: IP address to bind (default: "127.0.0.1")
    /// - SERVICE_PORT: Port to bind (default: "8080")
    pub fn from_env() -> Self {
        let ip = env::var("SERVICE_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "8080".to_string());

        Self { ip, port }
    }

    /// Get the bind address as "ip:port"
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_bind_address_from_ip_and_port() {
        // Arrange
        let config = ServerConfig {
            ip: "127.0.0.1".to_string(),
            port: "8080".to_string(),
        };

        // Act
        let address = config.bind_address();

        // Assert
        assert_eq!(address, "127.0.0.1:8080");
    }
}
