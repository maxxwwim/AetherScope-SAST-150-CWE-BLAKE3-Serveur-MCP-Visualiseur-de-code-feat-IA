---
id: arch-templates
layer: utility
max_lines_per_file: 300
allowed_imports: ["core_graph::*", "serde::*", "serde_json::*", "toml::*", "blake3::*", "std::*"]
forbidden_imports: ["security_linter::*", "ast_scanner::*", "ai_file_bridge::*", "desktop_shell::*", "axum::*", "tokio::*"]
security_rules:
  no_raw_sql: true
  no_env_access: true
---
# 🏗️ arch-templates — Hub de topologies & scaffolding multi-stack

Génère des squelettes de projet normés (11 dossiers) avec contrats `README.md` EARS pré-signés en BLAKE3 et `AETHER_AI_RULES.md`.

## 📦 Contenu

| Fichier | Rôle |
| :--- | :--- |
| `src/templates.rs` | Générateurs Pure Rust, Python (`uv`), Node (`pnpm`), Hybride Web3/MMO |

## 🛡️ Règles EARS

- WHEN a stack is selected THE SYSTEM SHALL generate the normalized folder tree for that stack.
- WHEN a folder is generated THE SYSTEM SHALL include its contract `README.md` with a BLAKE3 hash.
- IF the target folder already contains files THEN THE SYSTEM SHALL NOT overwrite them silently.
