use serde::{Serialize, Deserialize};

/// The config file structure for an iter project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterConfig {
    pub endpoint: String,
    pub project: Option<String>,
}

