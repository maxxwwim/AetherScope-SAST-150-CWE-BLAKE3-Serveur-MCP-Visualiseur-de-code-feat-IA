use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StackType {
    PureRust,
    PurePythonUv,
    PureNodePnpm,
    HybridMmoWeb3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TopologyMode {
    MonorepoLocal,
    Dissociated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyManifest {
    pub path: String,
    pub role: String,
    #[serde(default)]
    pub remote_sync: bool,
    pub remote_endpoint: Option<String>,
    pub readme_blake3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailSettings {
    #[serde(default = "default_true")]
    pub block_forbidden_imports: bool,
    #[serde(default = "default_true")]
    pub enforce_ears_syntax: bool,
    #[serde(default = "default_true")]
    pub sast_strict_mode: bool,
    #[serde(default = "default_true")]
    pub require_signature_on_contract_change: bool,
}

fn default_true() -> bool {
    true
}

impl Default for GuardrailSettings {
    fn default() -> Self {
        Self {
            block_forbidden_imports: true,
            enforce_ears_syntax: true,
            sast_strict_mode: true,
            require_signature_on_contract_change: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub version: u32,
    pub project_id: String,
    pub stack_type: StackType,
    pub topology_mode: TopologyMode,
    pub galaxies: HashMap<String, GalaxyManifest>,
    #[serde(default)]
    pub guardrails: GuardrailSettings,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        let mut galaxies = HashMap::new();
        galaxies.insert(
            "contracts".to_string(),
            GalaxyManifest {
                path: "./contracts".to_string(),
                role: "Strict schemas, API routes, WS packet definitions and Web3 ABIs".to_string(),
                remote_sync: false,
                remote_endpoint: None,
                readme_blake3: None,
            },
        );
        galaxies.insert(
            "client".to_string(),
            GalaxyManifest {
                path: "./client".to_string(),
                role: "Presentation, UI forms and local states".to_string(),
                remote_sync: false,
                remote_endpoint: None,
                readme_blake3: None,
            },
        );
        galaxies.insert(
            "server".to_string(),
            GalaxyManifest {
                path: "./server".to_string(),
                role: "Domain, application services and API endpoints".to_string(),
                remote_sync: false,
                remote_endpoint: None,
                readme_blake3: None,
            },
        );
        galaxies.insert(
            "data-nodes".to_string(),
            GalaxyManifest {
                path: "./data-nodes".to_string(),
                role: "Database migrations, models and caching layers".to_string(),
                remote_sync: false,
                remote_endpoint: None,
                readme_blake3: None,
            },
        );

        Self {
            version: 2,
            project_id: "aether_project".to_string(),
            stack_type: StackType::HybridMmoWeb3,
            topology_mode: TopologyMode::MonorepoLocal,
            galaxies,
            guardrails: GuardrailSettings::default(),
        }
    }
}
