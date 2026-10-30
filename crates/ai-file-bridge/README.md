---
id: ai-file-bridge
layer: infrastructure
max_lines_per_file: 450
allowed_imports: ["core_graph::*", "security_linter::*", "notify::*", "tokio::*", "serde::*", "serde_json::*", "chrono::*", "colored::*", "std::*"]
forbidden_imports: ["ast_scanner::*", "arch_templates::*", "desktop_shell::*", "axum::*"]
security_rules:
  no_raw_sql: true
---
# 🌉 ai-file-bridge — Passerelle IA « Zero-Shell »

Pont de fichiers temps réel entre le moteur et les agents IA : surveillance du disque, rapports Markdown, gestion des tâches et protocole de handshake.

## 📦 Contenu

| Fichier | Rôle |
| :--- | :--- |
| `src/watcher.rs` | `FileWatcherBridge` : surveillance réactive (`notify`, < 50 ms) |
| `src/audit_writer.rs` | `AuditWriter` : `.aether/LIVE_AUDIT.md`, `ACTIVE_TARGET.md`, prompts système |
| `src/tasks.rs` | `TaskManager` : CRUD de `.aether/tasks/<id>/task.json` + mini-`README.md`, `TASKS_SUMMARY.md` |
| `src/handshake.rs` | `HandshakePacker` : paquet XML `<aether_handshake version="2.0">` et preuve `[AETHER_ACK]` |

## 🛡️ Règles EARS

- WHEN a file is saved in the watched project THE SYSTEM SHALL regenerate `.aether/LIVE_AUDIT.md`.
- WHEN a task is created or updated THE SYSTEM SHALL persist `task.json` and refresh `.aether/TASKS_SUMMARY.md`.
- WHEN a handshake is requested THE SYSTEM SHALL emit a compact XML packet describing the target symbol and its contract status.
- IF the `.aether/` folder does not exist THEN THE SYSTEM SHALL create it before writing.
