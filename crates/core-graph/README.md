---
id: core-graph
layer: domain
max_lines_per_file: 400
allowed_imports: ["serde::*", "serde_json::*", "bitflags::*", "rayon::*", "std::*"]
forbidden_imports: ["security_linter::*", "ast_scanner::*", "ai_file_bridge::*", "arch_templates::*", "desktop_shell::*", "tokio::*", "axum::*"]
security_rules:
  no_raw_sql: true
  no_env_access: true
  deterministic_only: true
---
# 🧬 core-graph — Modèle de données & géométrie du graphe

Cœur **pur** du moteur : aucune I/O, aucun réseau, aucune variable d'environnement. Tous les autres crates en dépendent ; il ne dépend d'aucun d'eux.

## 📦 Contenu

| Fichier | Rôle |
| :--- | :--- |
| `src/bubble.rs` | `SymbolBubble`, `SymbolKind`, `Point2D`, `GalaxyNode`, métadonnées AST |
| `src/edge.rs` | `AffiliationEdge`, `EdgeType`, `CategorizedLiaisons`, `FilterAffiliation` (bitflags `u32`) |
| `src/graph.rs` | `AetherGraph`, `GraphStats` (degrés, métriques) |
| `src/layout.rs` | `compute_all_layouts` : 4 layouts (Obsidian, System, Flow DAG, UML) |
| `src/manifest.rs` | `ProjectManifest`, `GalaxyManifest`, `StackType`, `TopologyMode` |

## 🛡️ Règles EARS

- WHEN a layout is computed THE SYSTEM SHALL return the same positions for the same input graph.
- WHEN an edge filter is applied THE SYSTEM SHALL evaluate the `FilterAffiliation` bitmask in O(1) without reallocation.
- IF a crate other than `core-graph` is imported by this crate THEN THE SYSTEM SHALL report a `RULE_FORBIDDEN_IMPORT` violation.
- WHEN a file of this folder exceeds `max_lines_per_file` THE SYSTEM SHALL report a `RULE_MAX_LINES` violation.

## 🔌 API publique

```rust
pub fn compute_all_layouts(graph: &AetherGraph) -> LayoutPositions
```
