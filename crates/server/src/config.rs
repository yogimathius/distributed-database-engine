use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn new(port: u16) -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}