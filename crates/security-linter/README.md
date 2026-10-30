---
id: security-linter
layer: domain
max_lines_per_file: 4500
allowed_imports: ["core_graph::*", "serde::*", "serde_json::*", "serde_yaml::*", "blake3::*", "regex::*", "std::*"]
forbidden_imports: ["ast_scanner::*", "ai_file_bridge::*", "arch_templates::*", "desktop_shell::*", "axum::*", "tokio::*"]
security_rules:
  no_raw_sql: true
  no_env_access: true
  deterministic_only: true
---
# 🛡️ security-linter — Moteur SAST, lexer & contrats BLAKE3

Audit statique temps réel : 150+ règles CWE / Zero-Day, masquage lexical anti-faux-positifs, routage des règles par extension de fichier, et parsing des contrats `README.md` (frontmatter YAML + EARS + empreinte BLAKE3).

## 📦 Contenu

| Fichier | Rôle |
| :--- | :--- |
| `src/contract.rs` | `ReadmeContract::parse`, `ReadmeFrontmatter`, `EarsRequirement`, `verify_lock` |
| `src/lexer.rs` | Masquage des commentaires et littéraux, troncature de `#[cfg(test)]` |
| `src/linter.rs` | Catalogue central des règles, routage typé par extension, orchestrateur d'audit |
| `src/rules.rs` | Règles DeFi, vaults, oracles, taint tracking |
| `src/deep_rules.rs` | Règles bas niveau (Rust LLVM/UB/ABI, ZK-SNARKs, runtimes VM) |
| `src/php_rules.rs` | 25 règles PHP (SQLi, RCE, désérialisation, LFI/RFI, SSRF, SSTI…) |
| `tests/` | Tests de règles et `self_scan.rs` (le linter s'audite lui-même) |

> [!NOTE]
> `linter.rs` dépasse largement le budget standard de 300 lignes : le plafond de ce dossier est relevé en conséquence et un découpage par famille de CWE est recommandé.

## 🛡️ Règles EARS

- WHEN a source file is audited THE SYSTEM SHALL mask comments and non-relevant string literals before evaluating syntactic rules.
- WHEN a rule declares its `target_file_types` THE SYSTEM SHALL run it only on files with a matching extension.
- WHEN a contract `README.md` has a hash different from the one in `aether.lock.toml` THE SYSTEM SHALL raise the `BLAKE3_TAMPERED` violation.
- WHEN the same input is audited twice THE SYSTEM SHALL return identical findings.
- IF a contract frontmatter is not valid YAML THEN THE SYSTEM SHALL return an explicit parse error naming the folder.

## 🔌 API publique

```rust
pub fn parse(folder_path: &str, content: &str) -> Result<ReadmeContract, String>
pub fn verify_lock(&self, expected_hash: &str) -> bool
```
