---
id: ast-scanner
layer: application
max_lines_per_file: 3900
allowed_imports: ["core_graph::*", "security_linter::*", "walkdir::*", "rayon::*", "regex::*", "serde::*", "serde_json::*", "toml::*", "std::*"]
forbidden_imports: ["ai_file_bridge::*", "arch_templates::*", "desktop_shell::*", "axum::*", "tokio::*"]
security_rules:
  no_raw_sql: true
  no_env_access: true
---
# 🔍 ast-scanner — Scanner AST polyglotte multithread

Parcourt un projet (`walkdir` + `rayon`), extrait symboles, relations et routes, puis alimente `core-graph` et déclenche l'audit de `security-linter`.

## 📦 Contenu

| Fichier | Rôle |
| :--- | :--- |
| `src/scanner.rs` | `ProjectScanner` : extraction par regex `OnceLock` (Rust, Java, PHP, TS/JS, Python, C/C++, Smart Contracts, Markdown) |
| `src/lib.rs` | Ré-exports publics |

> [!NOTE]
> `scanner.rs` dépasse le budget standard : le plafond est relevé, un découpage par langage est recommandé.

## 🛡️ Règles EARS

- WHEN a project folder is scanned THE SYSTEM SHALL analyze files in parallel and build an `AetherGraph`.
- WHEN a file matches a user exclusion THE SYSTEM SHALL skip it entirely.
- WHEN a source file is parsed THE SYSTEM SHALL extract classes, traits, functions, routes and docstrings.
- IF a file cannot be read THEN THE SYSTEM SHALL continue the scan without aborting.
