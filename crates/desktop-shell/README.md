---
id: desktop-shell
layer: api
max_lines_per_file: 1100
allowed_imports: ["core_graph::*", "ast_scanner::*", "arch_templates::*", "security_linter::*", "ai_file_bridge::*", "axum::*", "tokio::*", "clap::*", "wry::*", "tao::*", "rfd::*", "serde::*", "serde_json::*", "std::*"]
forbidden_imports: []
security_rules:
  no_raw_sql: true
  untrusted_client_input: true
---
# 🖥️ desktop-shell — Exécutables natifs, API locale & serveur MCP

Point d'entrée de l'application. Produit deux binaires :

| Binaire | Source | Rôle |
| :--- | :--- | :--- |
| `aether-engine` | `src/main.rs` | Fenêtre native (tao/wry), serveur Axum sur `127.0.0.1:3030`, API REST locale |
| `aetherscope-mcp` | `src/mcp_main.rs` | Serveur MCP JSON-RPC 2.0 sur stdio (9 outils) |

## 📦 Contenu

| Fichier | Rôle |
| :--- | :--- |
| `src/main.rs` | Fenêtre WebView2, routes Axum, options CLI |
| `src/api_routes.rs` | DTOs, signature BLAKE3, scaffolding, dialogues OS natifs |
| `src/local_guard.rs` | Garde anti-CSRF / DNS rebinding (`Host` et `Origin` de bouclage) |
| `src/mcp.rs` | Implémentation des outils MCP avec repli direct sur disque |
| `src/viewer.rs` | Export HTML standalone |
| `assets/` | Cockpit frontend (voir son README) |

## 🛡️ Règles EARS

- WHEN a request reaches the local server THE SYSTEM SHALL reject it if its `Host` is not a loopback address with the server port.
- WHEN a request carries a foreign `Origin` THE SYSTEM SHALL reject it.
- WHEN the cockpit is closed and an MCP tool is called THE SYSTEM SHALL fall back to direct disk access instead of failing.
- WHEN a payload comes from the client THE SYSTEM SHALL treat it as untrusted and validate it before use.
