use super::{cors_config, server_config::ServerConfig};
use poem::middleware::Cors;

pub struct AppConfig {
    pub server: ServerConfig,
    pub cors: Cors,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig::from_env(),
            cors: cors_config::init_cors(),
        }
    }
}
