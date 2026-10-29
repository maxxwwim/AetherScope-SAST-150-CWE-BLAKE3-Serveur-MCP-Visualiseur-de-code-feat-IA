use chrono::Local;
use core_graph::bubble::SymbolBubble;
use security_linter::contract::ReadmeContract;
use security_linter::linter::LinterViolation;
use std::fs;
use std::path::Path;

pub struct AuditWriter;

impl AuditWriter {
    /// Écrit .aether/LIVE_AUDIT.md après chaque sauvegarde interceptée par le FileWatcher
    pub fn write_live_audit(project_root: &Path, violations: &[LinterViolation], audited_files_count: usize) -> std::io::Result<()> {
        let aether_dir = project_root.join(".aether");
        fs::create_dir_all(&aether_dir)?;

        let audit_file = aether_dir.join("LIVE_AUDIT.md");
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut content = String::new();
        content.push_str("# 🛡️ AETHER LIVE AUDIT — SYSTEM HEALTH\n\n");
        content.push_str(&format!("> **Timestamp :** `{}` | **Fichiers inspectés :** `{}`\n\n", now, audited_files_count));

        let security_violations: Vec<&LinterViolation> = violations
            .iter()
            .filter(|v| v.rule_id != "RULE_MAX_LINES" && v.severity != "WARNING")
            .collect();

        if violations.is_empty() || (security_violations.is_empty() && violations.iter().all(|v| v.rule_id == "RULE_MAX_LINES")) {
            content.push_str("## ✅ ARCH_OK : Zéro violation architecturale ou de sécurité\n\n");
            content.push_str("- Tous les imports respectent les contrats de dossier `README.md`.\n");
            content.push_str("- Aucune injection SQL brute ni secret détecté dans le code.\n");
            content.push_str("- Les budgets de complexité et de lignes sont respectés.\n\n");
            content.push_str("*(Ce fichier est mis à jour en temps réel à chaque `Ctrl+S` par le File-Watcher AetherScope)*\n");
        } else {
            content.push_str(&format!("## ❌ {} VIOLATION(S) DÉTECTÉE(S)\n\n", violations.len()));
            content.push_str("| Sévérité | Fichier | Ligne | Règle | Détail & Remédiation |\n");
            content.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

            for v in violations {
                let badge = match v.severity.as_str() {
                    "CRITICAL" => "🔴 CRITIQUE",
                    "HIGH" => "🟠 HAUT",
                    "MEDIUM" => "🟡 MOYEN",
                    _ => "⚪ AVERTISSEMENT",
                };
                let tainted = if let (Some(data), Some(ty)) = (&v.transmitted_data, &v.transmitted_type) {
                    format!("<br>📦 *Donnée :* `{}` (*Type :* `{}`)", data, ty)
                } else {
                    String::new()
                };
                content.push_str(&format!(
                    "| {} | `{}` | `{}` | `{}`<br><small>{}</small> | **{}**{}<br>*Fix:* {} |\n",
                    badge, v.file_path, v.line_number, v.rule_id, v.cve_id, v.message, tainted, v.remediation
                ));
            }
            if !security_violations.is_empty() {
                content.push_str("\n> ⚠️ **Consigne pour l'Agent IA (Cursor/Antigravity/Claude) :** Corrige immédiatement ces violations de sécurité avant de continuer.\n");
            }
        }

        fs::write(audit_file, content)
    }

    /// Écrit .aether/ACTIVE_TARGET.md lorsqu'une bulle est ciblée dans le cockpit WebGL
    pub fn write_active_target(
        project_root: &Path,
        symbol: &SymbolBubble,
        callers: &[String],
        dependencies: &[String],
        contract: Option<&ReadmeContract>,
    ) -> std::io::Result<()> {
        let mut in_cat = core_graph::edge::CategorizedLiaisons::default();
        in_cat.internal_calls = callers.to_vec();
        let mut out_cat = core_graph::edge::CategorizedLiaisons::default();
        out_cat.internal_calls = dependencies.to_vec();
        Self::write_active_target_categorized(project_root, symbol, &in_cat, &out_cat, contract)
    }

    /// Écrit .aether/ACTIVE_TARGET.md avec les liaisons structurées par catégories légères
    pub fn write_active_target_categorized(
        project_root: &Path,
        symbol: &SymbolBubble,
        incoming: &core_graph::edge::CategorizedLiaisons,
        outgoing: &core_graph::edge::CategorizedLiaisons,
        contract: Option<&ReadmeContract>,
    ) -> std::io::Result<()> {
        let aether_dir = project_root.join(".aether");
        fs::create_dir_all(&aether_dir)?;

        let target_file = aether_dir.join("ACTIVE_TARGET.md");
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut content = String::new();
        content.push_str("# 🎯 AETHER ACTIVE TARGET CONTEXT\n\n");
        content.push_str(&format!("> Synchronisé le `{}` depuis le cockpit AetherScope\n\n", now));
        content.push_str(&format!("- **Symbole :** `{}`\n", symbol.name));
        content.push_str(&format!("- **Fichier :** `{}:{}`\n", symbol.file_path, symbol.line_number));
        content.push_str(&format!("- **Galaxie :** `{}` | **Couche :** `{}`\n", symbol.galaxy, symbol.layer));
        content.push_str(&format!("- **Type :** `{:?}`\n\n", symbol.kind));

        content.push_str("## 📐 Dépendances & Flux (Synthèse Catégorisée)\n\n");
        content.push_str(&format!("### 📥 FLUX AMONT / APPELANTS (In-Degree: {})\n", incoming.total()));
        content.push_str(&incoming.to_compact_markdown("amont", 4));

        content.push_str(&format!("\n### 📤 FLUX AVAL / DÉPENDANCES & LIAISONS (Out-Degree: {})\n", outgoing.total()));
        content.push_str(&outgoing.to_compact_markdown("aval", 4));

        if let Some(c) = contract {
            content.push_str("\n## 🔒 Contrat du dossier (`README.md`)\n\n");
            content.push_str(&format!("- **Imports autorisés :** `{:?}`\n", c.frontmatter.allowed_imports));
            content.push_str(&format!("- **Imports INTERDITS :** `{:?}`\n", c.frontmatter.forbidden_imports));
            content.push_str(&format!("- **Lignes max :** `{}`\n\n", c.frontmatter.max_lines_per_file));

            if !c.ears_requirements.is_empty() {
                content.push_str("### 📋 Exigences EARS\n");
                for (idx, r) in c.ears_requirements.iter().enumerate() {
                    content.push_str(&format!("{}. **WHEN** {} **THE SYSTEM SHALL** {}\n", idx + 1, r.trigger, r.system_shall));
                }
            }
        }

        // Tâches actives associées à ce symbole ou ce fichier
        let all_tasks = crate::tasks::TaskManager::list_tasks(project_root);
        let relevant_tasks: Vec<&crate::tasks::TaskItem> = all_tasks
            .iter()
            .filter(|t| {
                t.status != crate::tasks::TaskStatus::Done
                    && t.status != crate::tasks::TaskStatus::Cancelled
                    && (t.target_symbol.as_deref() == Some(&symbol.name)
                        || t.target_file.as_deref() == Some(&symbol.file_path)
                        || t.target_symbol.as_deref() == Some(&symbol.id))
            })
            .collect();

        if !relevant_tasks.is_empty() {
            content.push_str("\n## 📋 TÂCHES COPILOTE ASSOCIÉES À CETTE CIBLE\n\n");
            for t in relevant_tasks {
                content.push_str(&format!(
                    "- {} `{}` **{}** (Statut: {}, Progrès: {}%)\n",
                    t.kind.icon(),
                    t.id,
                    t.title,
                    t.status.badge(),
                    t.progress
                ));
                if !t.readme_summary.is_empty() {
                    content.push_str(&format!("  *Résumé avancement :* {}\n", t.readme_summary));
                }
            }
        }

        fs::write(target_file, content)
    }

    /// Détecte la stack réelle du projet en analysant les manifestes et lockfiles
    pub fn detect_project_stack(project_root: &Path) -> &'static str {
        if project_root.join("aether.lock.toml").exists() {
            if let Ok(lock) = fs::read_to_string(project_root.join("aether.lock.toml")) {
                let lock_lower = lock.to_lowercase();
                if lock_lower.contains("hybrid") || lock_lower.contains("mmo") || lock_lower.contains("web3") {
                    return "hybrid";
                }
            }
        }
        if (project_root.join("Cargo.toml").exists() || project_root.join("Cargo.lock").exists())
            && (project_root.join("package.json").exists() || project_root.join("client").exists())
        {
            "hybrid"
        } else if project_root.join("Cargo.toml").exists() || project_root.join("Cargo.lock").exists() {
            "rust"
        } else if project_root.join("pyproject.toml").exists() || project_root.join("requirements.txt").exists() {
            "python"
        } else if project_root.join("package.json").exists() || project_root.join("pnpm-lock.yaml").exists() {
            "ts"
        } else {
            "hybrid"
        }
    }

    /// Génère le prompt système optimisé spécifique au projet et à sa stack
    pub fn generate_optimized_system_prompt(
        project_root: &Path,
        project_name: &str,
        stack_str: &str,
        server_port: u16,
    ) -> String {
        let p_str = project_root.to_string_lossy();
        let (stack_title, stack_arch, stack_contracts) = match stack_str.to_lowercase().as_str() {
            "rust" => (
                "🦀 Pure Rust (Cargo Workspace Hexagonal)",
                "Cargo Workspace : `crates/domain` (100% pur), `crates/server-axum`, `crates/client-ui`, `crates/db-adapter`",
                "Traits Rust (`RepositoryPort`), DTOs partagés Serde et schémas RPC stricts",
            ),
            "python" => (
                "🐍 Pure Python (Modular Clean Monorepo uv)",
                "`uv` Workspace : `contracts/` (Pydantic v2), `server/api/` (FastAPI), `server/domain/` (Pure logic), `data-nodes/` (SQLAlchemy)",
                "Schémas Pydantic v2 dans `contracts/` validant tous les échanges",
            ),
            "ts" | "node" => (
                "🟢 Node.js TypeScript (pnpm Workspace)",
                "`pnpm` Workspace : `contracts/` (Zod), `server/` (Hono / tRPC), `client/` (Vite UI), `data-nodes/` (Drizzle / Prisma)",
                "Schémas Zod et routeurs tRPC de bout en bout",
            ),
            _ => (
                "⚡ Hybride Web3 & Temps Réel MMO",
                "Client PixiJS/TS + Serveur Rust WebSocket + Adaptateurs Data-Nodes (PostgreSQL / Redis)",
                "Paquets binaires WebSocket + ABIs Smart Contracts dans `contracts/`",
            ),
        };

        format!(
            r#"# 🪐 AETHERSCOPE V2 — DIRECTIVES SYSTÈME COPILOTE IA (ARCHITECTURE & PAIR-PROGRAMMING)

> **Projet :** `{}`  
> **Racine :** `{}`  
> **Stack Détectée :** {}  
> **Architecture Active :** {}  
> **Serveur Local AetherScope :** `http://127.0.0.1:{}`  

Tu opères en binôme direct avec le développeur et le moteur d'intelligence architecturale **AetherScope V2**.
Le graphe 3D WebGL 2 tourne en continu sur le port `{}` et audite chaque modification du codebase.

---

## 🚀 1. DIAGNOSTIC INITIAL AU LANCEMENT DE CONVERSATION
Dès ton premier message ou au démarrage d'une session :
1. **Émets un diagnostic de santé d'architecture & d'avancement des tâches :**
   - Appelle `aetherscope_get_graph_summary` (ou lis `.aether/LIVE_AUDIT.md`) et `aetherscope_list_tasks` (ou lis `.aether/TASKS_SUMMARY.md`).
   - Annonce en 2-3 lignes chirurgicales l'état du système :
     `🪐 [AETHER_STATUS : Nœuds=<N>, Flux=<F>, Sécurité=<OK | X violations>, Tâches=<T actives>, Serveur=Connecté]`
2. **Si l'utilisateur mentionne un module, symbole ou objectif :**
   - Appelle immédiatement `aetherscope_get_active_target(symbol_id)` pour extraire le contexte ciblé, les flux catégorisés, les tâches associées et les invariants EARS.
   - Aligne la caméra WebGL via `aetherscope_teleport_camera` (ou `POST /api/ai/telemetry`).

---

## 🛠️ 2. OUTILLAGE MCP COMPLET (AETHERSCOPE MCP ENGINE)
Utilise systématiquement les outils MCP mis à disposition :
- **`aetherscope_teleport_camera(target, action, details)`** :
  Déplace instantanément et avec fluidité la caméra 3D sur la bulle/dossier cible, déclenche le faisceau quantique et éclaire le fil d'Ariane stellaire en direct pour le développeur.
- **`aetherscope_get_graph_summary(path)`** :
  Fournit la vue globale : volumes de nœuds, relations, répartition des modules et détection proactive des violations CVE.
- **`aetherscope_get_active_target(symbol_id)`** :
  Génère le contexte chirurgical allégé avec les **flux catégorisés** et les tâches en cours sur la cible :
  - 📝 Actions UI / Submits
  - 🌐 Endpoints & Requêtes HTTP
  - ⚡ Temps Réel & Packets WebSocket
  - 🗄️ Persistance DB / SQL
  - 🏛️ Dépendances Transverses & Contrats
  - 📦 Appels Internes
  - 🚨 Violations & Alertes
  - 📋 Tâches & mini-READMEs associés
- **`aetherscope_create_task(title, kind, target_symbol, target_file, priority, initial_readme)`** :
  Crée une tâche (fonction, recherche, sécurité, refactor) avec son mini-README d'avancement consultable par le modèle et l'utilisateur.
- **`aetherscope_update_task(task_id, status, progress, progress_note, full_readme)`** :
  Met à jour le statut (`pending`, `in_progress`, `blocked`, `done`), le pourcentage d'avancement et ajoute des notes de progrès ou coche les étapes du README.
- **`aetherscope_list_tasks(status_filter)`** :
  Liste toutes les tâches du projet avec leur état, auteur et résumé d'avancement.
- **`aetherscope_scan_project(path)`** :
  Déclenche un rescan ultrarapide (< 100ms) pour rafraîchir l'AST et les métriques du graphe 3D.
- **`aetherscope_send_chat_message(text, thought, target_symbol)`** :
  Envoie un debrief technique ou compte-rendu directement dans le chat intégré du cockpit AetherScope.
- **`aetherscope_get_chat_history()`** :
  Lit l'historique récent des échanges et demandes passées dans le cockpit.

---

## 📋 3. GESTION DES TÂCHES & MINI-README D'AVANCEMENT
1. **Création Systématique :** Dès qu'une modification touche plus d'une fonction, requiert une recherche/audit ou une refactorisation :
   - Crée une tâche via `aetherscope_create_task` (ou modifie `.aether/tasks/<task_id>/README.md`).
   - Choisis le bon type : `function` (code), `research` (exploration/benchmark), `security_fix` (CVE), `arch_refactor` (structure), ou `other`.
2. **Suivi d'Avancement :**
   - Utilise le mini-README de la tâche pour documenter le plan, cocher les cases au fur et à mesure et lister les résultats d'investigation.
   - Mets à jour la progression via `aetherscope_update_task` au fil de tes actions.

---

## 🔒 4. ZERO-SHELL PROTOCOLE DE GOUVERNANCE
1. **Avant d'écrire ou modifier du code :**
   - Lis `.aether/ACTIVE_TARGET.md` (ou réponse de `aetherscope_get_active_target`).
   - Lis le `README.md` du dossier cible pour connaître les règles EARS et les invariants.
   - Ne modifie JAMAIS un `README.md` pour contourner une règle : le hash cryptographique BLAKE3 est scellé dans `aether.lock.toml`.
2. **Après chaque sauvegarde (`Ctrl+S`) :**
   - Le File-Watcher Rust réanalyse l'AST en < 50ms et régénère `.aether/LIVE_AUDIT.md`.
   - Si `.aether/LIVE_AUDIT.md` contient `❌ VIOLATION`, corrige immédiatement la ligne signalée avant de continuer.

---

## 🏛️ 5. INVARIANTS PAR COUCHE & SÉPARATION STRICTE
- `contracts/` : {}
- `client/` : Rendu graphique, formulaires («Form»), état réactif. Ne doit JAMAIS importer `server/domain` ni `data-nodes`.
- `server/domain/` : **100% pur, déterministe, zéro I/O**. Interdiction formelle de requêtes SQL brutes, d'accès réseau ou de variables d'environnement (`process.env` / `std::env::var`).
- `server/application/` : Orchestrations des cas d'usage reliant le domaine aux interfaces de persistance.
- `server/api/` : Contrôleurs d'entrée (REST / WebSocket) validant tout payload via `contracts/`.
- `data-nodes/` : Adaptateurs de persistance (PostgreSQL / SQLite / Redis) isolables en microservices.

---

## ⚡ 6. CONSIGNES DE RÉPONSE OPTIMISÉE
- **Concision & Densité :** Réponse technique, chirurgicale et directe. Zéro bavardage inutile.
- **Diffs Chirurgicaux :** Ne réécris jamais un fichier complet si seules quelques lignes changent. Fournis uniquement les extraits ciblés.
- **Règle d'Atomicité (< 35 lignes) :** Découpe le code en fonctions atomiques pures de moins de 35 lignes maximum (particulièrement dans `domain/`).
- **Preuve de Lecture [AETHER_ACK] :** Démarre tes propositions techniques par :
  `[AETHER_ACK: cible=<symbole>, couche=<layer>, imports_interdits=<liste>]`
"#,
            project_name, p_str, stack_title, stack_arch, server_port, server_port, stack_contracts
        )
    }

    /// Génère le fichier maître AETHER_AI_RULES.md à la racine du projet
    pub fn ensure_master_rules_file(project_root: &Path) -> std::io::Result<()> {
        let rules_file = project_root.join("AETHER_AI_RULES.md");
        let proj_name = project_root
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Project".to_string());
        let stack_str = Self::detect_project_stack(project_root);
        let content = Self::generate_optimized_system_prompt(project_root, &proj_name, stack_str, 3030);
        fs::write(rules_file, content)
    }
}
