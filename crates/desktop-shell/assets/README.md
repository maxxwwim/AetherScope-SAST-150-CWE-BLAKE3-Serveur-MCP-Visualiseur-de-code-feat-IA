---
id: desktop-shell-assets
layer: ui
max_lines_per_file: 3500
allowed_imports: []
forbidden_imports: ["server/domain::*", "data-nodes::*"]
security_rules:
  untrusted_client_input: true
---
# 🎛️ assets — Cockpit frontend WebGL 2

Frontend modulaire, **sans framework**, embarqué dans le binaire. Il communique uniquement avec l'API REST locale (`/api/...`).

## 📦 Contenu

| Fichier | Rôle |
| :--- | :--- |
| `index.html` | Squelette DOM, topbar, inspecteur, modales |
| `styles.css` | Design system cosmique |
| `state_api.js` | Store réactif, `AudioSys`, client REST `AetherAPI` |
| `webgl_shaders_physics.js` | Shaders GLSL SDF et physique Coulomb/Hooke |
| `webgl_renderer.js` | Rendu instancié, membranes à 12 lobes, photons |
| `ui_navigation.js` | Caméra inertielle, zoom LoD, sélection |
| `ui_inspector_docs.js` | Inspecteur 4 onglets, éditeur Markdown, sceau BLAKE3 |
| `ui_vision_modes.js` | X-Ray, Dead Code, Heatmap, CVE triage, scope Bug Bounty |
| `ui_ai_modals.js` | Omnibox, chat Copilot, tâches, télémétrie, MCP |

## 🛡️ Règles EARS

- WHEN data coming from the project or the API is rendered THE SYSTEM SHALL escape it before inserting it into the DOM.
- WHEN the physics simulation does not move any node THE SYSTEM SHALL NOT update the vertex buffers.
- IF WebGL 2 is unavailable THEN THE SYSTEM SHALL display an explicit error message.
