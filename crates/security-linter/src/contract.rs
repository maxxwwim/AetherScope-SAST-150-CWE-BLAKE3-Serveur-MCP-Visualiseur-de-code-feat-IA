use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityRules {
    #[serde(default)]
    pub no_raw_sql: bool,
    #[serde(default)]
    pub no_env_access: bool,
    #[serde(default)]
    pub untrusted_client_input: bool,
    #[serde(default)]
    pub deterministic_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeFrontmatter {
    pub id: String,
    pub layer: String,
    #[serde(default = "default_max_lines")]
    pub max_lines_per_file: usize,
    #[serde(default)]
    pub allowed_imports: Vec<String>,
    #[serde(default)]
    pub forbidden_imports: Vec<String>,
    #[serde(default)]
    pub security_rules: SecurityRules,
}

fn default_max_lines() -> usize {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarsRequirement {
    pub raw: String,
    pub trigger: String, // "WHEN ..."
    pub system_shall: String, // "THE SYSTEM SHALL ..."
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeContract {
    pub folder_path: String,
    pub frontmatter: ReadmeFrontmatter,
    pub ears_requirements: Vec<EarsRequirement>,
    pub exported_signatures: Vec<String>,
    pub blake3_hash: String,
}

impl ReadmeContract {
    /// Parse un README.md avec frontmatter YAML et syntaxe EARS
    pub fn parse(folder_path: &str, content: &str) -> Result<Self, String> {
        let blake3_hash = blake3::hash(content.as_bytes()).to_hex().to_string();

        let mut frontmatter_str = "";
        let mut body_str = content;

        if content.starts_with("---") {
            if let Some(end_idx) = content[3..].find("---") {
                frontmatter_str = &content[3..end_idx + 3];
                body_str = &content[end_idx + 6..];
            }
        }

        let frontmatter: ReadmeFrontmatter = if !frontmatter_str.trim().is_empty() {
            serde_yaml::from_str(frontmatter_str)
                .map_err(|e| format!("YAML frontmatter parse error in {}: {}", folder_path, e))?
        } else {
            ReadmeFrontmatter {
                id: folder_path.replace(['/', '\\'], "-"),
                layer: "utility".to_string(),
                max_lines_per_file: 300,
                allowed_imports: Vec::new(),
                forbidden_imports: Vec::new(),
                security_rules: SecurityRules::default(),
            }
        };

        // Extraction des règles EARS (WHEN ... THE SYSTEM SHALL ...)
        let mut ears = Vec::new();
        for line in body_str.lines() {
            let trimmed = line.trim();
            if trimmed.to_uppercase().contains("WHEN ") && trimmed.to_uppercase().contains("THE SYSTEM SHALL") {
                let upper = trimmed.to_uppercase();
                if let (Some(w_idx), Some(s_idx)) = (upper.find("WHEN "), upper.find("THE SYSTEM SHALL")) {
                    if s_idx > w_idx {
                        let trigger = trimmed[w_idx + 5..s_idx].trim().trim_end_matches(',').to_string();
                        let system_shall = trimmed[s_idx + 16..].trim().to_string();
                        ears.push(EarsRequirement {
                            raw: trimmed.to_string(),
                            trigger,
                            system_shall,
                        });
                    }
                }
            }
        }

        // Extraction des signatures de code dans les blocs de code markdown
        let mut signatures = Vec::new();
        let mut inside_code_block = false;
        for line in body_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                inside_code_block = !inside_code_block;
                continue;
            }
            if inside_code_block && (trimmed.starts_with("pub fn ") || trimmed.starts_with("export function ") || trimmed.starts_with("def ")) {
                signatures.push(trimmed.to_string());
            }
        }

        Ok(Self {
            folder_path: folder_path.to_string(),
            frontmatter,
            ears_requirements: ears,
            exported_signatures: signatures,
            blake3_hash,
        })
    }

    /// Vérifie si le hash BLAKE3 correspond au lockfile
    pub fn verify_lock(&self, expected_hash: &str) -> bool {
        self.blake3_hash == expected_hash
    }
}
