use core_graph::graph::AetherGraph;
use core_graph::manifest::{GalaxyManifest, ProjectManifest, StackType, TopologyMode};
use arch_templates::ArchScaffolder;
use ai_file_bridge::AuditWriter;
use security_linter::contract::ReadmeContract;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

#[derive(Serialize, Clone)]
pub struct ProjectItem {
    pub name: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct ProjectsResponse {
    pub projects: Vec<ProjectItem>,
    pub default_project: Option<String>,
}

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub path: String,
    #[serde(default)]
    pub exclusions: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsPayload {
    #[serde(default)]
    pub exclusions: Vec<String>,
    #[serde(default)]
    pub active_cve_rules: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cve_catalogue: Option<Vec<security_linter::CveRuleDef>>,
    /// Chemin réel de `aetherscope-mcp` (à côté de l'exécutable en cours) pour générer la config MCP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_binary_path: Option<String>,
}

/// Chemin absolu du binaire `aetherscope-mcp`, résolu relativement à l'exécutable courant.
pub fn resolve_mcp_binary_path() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let name = if cfg!(windows) { "aetherscope-mcp.exe" } else { "aetherscope-mcp" };
    Some(clean_path_string(&exe.parent()?.join(name)))
}

#[derive(Deserialize)]
pub struct ExportRequest {
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct ExportResponse {
    pub status: String,
    pub viewer_path: String,
    pub total_nodes: usize,
    pub total_edges: usize,
}

#[derive(Deserialize)]
pub struct SaveDocRequest {
    pub project_path: String,
    pub folder: String,
    pub file_name: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct SaveDocResponse {
    pub status: String,
    pub saved_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blake3_hash: Option<String>,
}

#[derive(Deserialize)]
pub struct SignContractsRequest {
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub folder: Option<String>,
}

#[derive(Serialize)]
pub struct SignContractsResponse {
    pub status: String,
    pub message: String,
    pub signed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blake3_hash: Option<String>,
    pub graph: AetherGraph,
}

#[derive(Deserialize)]
pub struct AiTargetRequest {
    pub symbol_id: String,
    #[serde(default)]
    pub action_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiTargetResponse {
    pub status: String,
    pub active_target_path: String,
    pub handshake_xml: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blake3_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_verified: Option<bool>,
    pub ears_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub galaxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docstring: Option<String>,
    #[serde(default)]
    pub callers: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categorized_incoming: Option<core_graph::edge::CategorizedLiaisons>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categorized_outgoing: Option<core_graph::edge::CategorizedLiaisons>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_doc_and_liaisons: Option<String>,
}

#[derive(Deserialize)]
pub struct InitProjectRequest {
    pub path: String,
    #[serde(default = "default_stack")]
    pub stack: String, // "rust" | "python" | "ts" | "hybrid"
}

fn default_stack() -> String {
    "hybrid".to_string()
}

#[derive(Serialize)]
pub struct InitProjectResponse {
    pub status: String,
    pub message: String,
    pub created_dirs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiTelemetryEvent {
    pub target: String,
    pub action: String,
    pub details: String,
    #[serde(default)]
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiChatMessage {
    pub id: String,
    pub sender: String,
    pub text: String,
    #[serde(default)]
    pub target_symbol: Option<String>,
    #[serde(default)]
    pub timestamp: u64,
}

#[derive(Deserialize)]
pub struct SendChatMessageRequest {
    pub text: String,
    #[serde(default)]
    pub sender: Option<String>,
    #[serde(default)]
    pub target_symbol: Option<String>,
}

#[derive(Serialize)]
pub struct ChatMessagesResponse {
    pub messages: Vec<AiChatMessage>,
    pub agent_status: String,
}

#[derive(Deserialize)]
pub struct CreateTaskApiRequest {
    pub title: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub target_symbol: Option<String>,
    #[serde(default)]
    pub target_file: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub initial_readme: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTaskApiRequest {
    pub task_id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub progress: Option<u8>,
    #[serde(default)]
    pub progress_note: Option<String>,
    #[serde(default)]
    pub full_readme: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteTaskApiRequest {
    pub task_id: String,
}

#[derive(Deserialize)]
pub struct CreateFileRequest {
    pub project_path: String,
    pub file_path: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct LaunchCliRequest {
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Serialize)]
pub struct CreateFileResponse {
    pub status: String,
    pub file_path: String,
    pub created_symbol: String,
    pub graph: AetherGraph,
    pub data: AetherGraph,
}

#[derive(Serialize)]
pub struct PickWorkspaceResponse {
    pub status: String,
    pub workspace: Option<String>,
    pub projects: Vec<ProjectItem>,
}

#[derive(Serialize)]
pub struct PickProjectResponse {
    pub status: String,
    pub project_path: Option<String>,
    pub graph: Option<AetherGraph>,
}

pub struct AppState {
    pub graph: RwLock<AetherGraph>,
    pub project_root: RwLock<PathBuf>,
    pub telemetry: RwLock<Option<AiTelemetryEvent>>,
    pub chat_messages: RwLock<Vec<AiChatMessage>>,
    pub graph_version: std::sync::atomic::AtomicU64,
    pub custom_exclusions: RwLock<Vec<String>>,
    pub active_cve_rules: RwLock<Option<Vec<String>>>,
}

pub fn clean_path_string(p: &Path) -> String {
    let s = p.to_string_lossy().to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        s
    }
}

pub fn discover_candidate_projects(workspace_root: &Path) -> Vec<ProjectItem> {
    let mut candidates = Vec::new();

    if let Ok(entries) = fs::read_dir(workspace_root) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with('.') && name != "target" && name != "node_modules" && name != "dist" {
                        candidates.push(ProjectItem {
                            name,
                            path: clean_path_string(&entry.path()),
                        });
                    }
                }
            }
        }
    }

    if let Some(parent) = workspace_root.parent() {
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let p_str = clean_path_string(&entry.path());
                        if !name.starts_with('.') && name != "target" && !candidates.iter().any(|c| c.path == p_str) {
                            candidates.push(ProjectItem {
                                name,
                                path: p_str,
                            });
                        }
                    }
                }
            }
        }
    }

    candidates.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    candidates
}

/// Calcule les hashes BLAKE3 des contrats README.md et met à jour aether.lock.toml
pub fn sign_project_contracts(base: &Path, folder_filter: Option<&str>) -> Result<(usize, Option<String>), String> {
    let lock_file = base.join("aether.lock.toml");
    let proj_name = base
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());

    let mut manifest: ProjectManifest = if let Ok(lock_str) = fs::read_to_string(&lock_file) {
        toml::from_str(&lock_str).unwrap_or_else(|_| {
            ArchScaffolder::create_manifest(&proj_name, StackType::HybridMmoWeb3, TopologyMode::MonorepoLocal)
        })
    } else {
        ArchScaffolder::create_manifest(&proj_name, StackType::HybridMmoWeb3, TopologyMode::MonorepoLocal)
    };

    let mut signed_count = 0usize;
    let mut last_hash = None;

    let norm_filter = folder_filter.map(|f| {
        f.trim_start_matches("./")
            .trim_start_matches('/')
            .replace('\\', "/")
    });

    for entry in walkdir::WalkDir::new(base)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "target" && name != "node_modules" && name != "dist"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.file_name().to_string_lossy().eq_ignore_ascii_case("README.md") {
            let rel_folder = entry
                .path()
                .parent()
                .and_then(|p| p.strip_prefix(base).ok())
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "root".to_string());

            if let Some(ref f) = norm_filter {
                if !f.is_empty() && f != &rel_folder && !(f == "." && rel_folder == "root") {
                    continue;
                }
            }

            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(contract) = ReadmeContract::parse(&rel_folder, &content) {
                    let hash = contract.blake3_hash.clone();
                    last_hash = Some(hash.clone());
                    signed_count += 1;

                    let key = if rel_folder == "root" {
                        "root".to_string()
                    } else {
                        rel_folder.clone()
                    };

                    if let Some(gal) = manifest.galaxies.get_mut(&key) {
                        gal.readme_blake3 = Some(hash);
                    } else {
                        manifest.galaxies.insert(
                            key,
                            GalaxyManifest {
                                path: if rel_folder == "root" {
                                    ".".to_string()
                                } else {
                                    format!("./{}", rel_folder)
                                },
                                role: format!("Signed contract for {}", rel_folder),
                                remote_sync: false,
                                remote_endpoint: None,
                                readme_blake3: Some(hash),
                            },
                        );
                    }
                }
            }
        }
    }

    if let Ok(lock_str) = toml::to_string_pretty(&manifest) {
        fs::write(&lock_file, lock_str).map_err(|e| e.to_string())?;
    }

    Ok((signed_count, last_hash))
}

pub fn init_project_structure(base: &Path, stack_str: &str) -> Vec<String> {
    let _ = fs::create_dir_all(base);

    let stack = match stack_str.to_lowercase().as_str() {
        "rust" => StackType::PureRust,
        "python" => StackType::PurePythonUv,
        "ts" | "node" => StackType::PureNodePnpm,
        _ => StackType::HybridMmoWeb3,
    };

    let proj_name = base.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "project".to_string());
    let manifest = ArchScaffolder::create_manifest(&proj_name, stack.clone(), TopologyMode::MonorepoLocal);

    let lock_file = base.join("aether.lock.toml");
    if let Ok(lock_str) = toml::to_string_pretty(&manifest) {
        let _ = fs::write(&lock_file, lock_str);
    }

    let (stack_title, stack_contract_note, stack_db_note) = match stack {
        StackType::PureRust => (
            "🦀 Rust (Cargo Workspace Hexagonal)",
            "Traits Rust stricts, DTOs partagés Serde et schémas RPC",
            "Trait RepositoryPort : bascule SQLite local (fichier) <-> Postgres distant par config",
        ),
        StackType::PurePythonUv => (
            "🐍 Python (Modular Clean Monorepo uv)",
            "Schémas Pydantic v2 stricts dans contracts/ validant I/O API et Workers",
            "SQLAlchemy 2 / SQLModel avec adaptateur SQLite local ou Postgres distant",
        ),
        StackType::PureNodePnpm => (
            "🟢 Node.js TypeScript (pnpm Workspace)",
            "Validation de bout en bout Zod + Routeurs tRPC / Hono",
            "Packages/db Drizzle ou Prisma isolable en microservice indépendant",
        ),
        StackType::HybridMmoWeb3 => (
            "⚡ Hybride Web3 & Temps Réel MMO",
            "Paquets WebSocket binaires, schémas RPC et ABIs Smart Contracts",
            "PostgreSQL State, Redis pub/sub temps réel et indexeurs On-Chain",
        ),
    };

    let structure = vec![
        ("contracts", format!("---\nid: contracts\nlayer: contracts\nmax_lines_per_file: 200\nallowed_imports: []\nforbidden_imports: [\"client::*\", \"server::*\"]\n---\n# 📜 Contrats Globaux & Interfaces Partagées\n\n> **Stack :** {}\n> **Rôle :** {}\n\n## 🛡️ Règles EARS\n- WHEN client issues API request THE SYSTEM SHALL validate schema.\n- WHEN server emits response THE SYSTEM SHALL guarantee type safety.\n", stack_title, stack_contract_note)),
        ("client", format!("---\nid: client\nlayer: ui\nmax_lines_per_file: 300\nallowed_imports: [\"contracts::*\"]\nforbidden_imports: [\"server::domain::*\", \"data-nodes::*\"]\n---\n# 💻 Couche Client (Interface & Présentation)\n\n> Rendu graphique, formulaires («Form») et gestion d'état local.\n\n- WHEN user interacts with view THE SYSTEM SHALL update reactive local state.\n")),
        ("client/ui", format!("---\nid: client-ui\nlayer: ui\nmax_lines_per_file: 250\nallowed_imports: [\"contracts::*\", \"client/state::*\"]\nforbidden_imports: [\"server::*\"]\n---\n# 🎨 Composants UI & Formulaires\n\nWHEN user submits form THE SYSTEM SHALL trigger client validation.\n")),
        ("client/state", format!("---\nid: client-state\nlayer: ui\nmax_lines_per_file: 200\nallowed_imports: [\"contracts::*\"]\nforbidden_imports: [\"server::domain::*\"]\n---\n# 🧠 Gestion d'État Client\n\nStockage réactif local et synchronisation des sessions.\n")),
        ("server", format!("---\nid: server\nlayer: domain\nmax_lines_per_file: 300\nallowed_imports: [\"contracts::*\"]\nforbidden_imports: [\"client::*\"]\nsecurity_rules:\n  no_raw_sql: true\n  no_env_access: true\n---\n# ⚡ Cœur Métier & API Serveur\n\n> Couche dissociable pouvant tourner en local ou sur serveur dédié.\n")),
        ("server/domain", format!("---\nid: server-domain\nlayer: domain\nmax_lines_per_file: 200\nallowed_imports: []\nforbidden_imports: [\"server/api::*\", \"client::*\", \"data-nodes::*\"]\nsecurity_rules:\n  deterministic_only: true\n---\n# 🏛️ Logique Métier Pure (Domain)\n\n100% pur, zéro dépendance I/O externe. Règles fondamentales immuables.\n\n- WHEN domain rule is evaluated THE SYSTEM SHALL execute deterministically without I/O.\n")),
        ("server/application", format!("---\nid: server-app\nlayer: action\nmax_lines_per_file: 250\nallowed_imports: [\"server/domain::*\", \"contracts::*\"]\nforbidden_imports: [\"client::*\"]\n---\n# ⚙️ Orchestration & Cas d'Usage\n\nServices applicatifs coordonnant le domaine et les adaptateurs.\n")),
        ("server/api", format!("---\nid: server-api\nlayer: action\nmax_lines_per_file: 250\nallowed_imports: [\"server/application::*\", \"contracts::*\"]\nforbidden_imports: [\"server/domain::*\"]\n---\n# 🌐 Entrées API & Contrôleurs RPC/WebSocket\n\nWHEN network packet received THE SYSTEM SHALL route through contracts schema.\n")),
        ("data-nodes", format!("---\nid: data-nodes\nlayer: database\nmax_lines_per_file: 300\nallowed_imports: [\"contracts::*\"]\nforbidden_imports: [\"client::*\"]\n---\n# 🗄️ Services & Nœuds de Données Dissociables\n\n> **Découplage :** {}\n", stack_db_note)),
        ("data-nodes/db-primary", format!("---\nid: db-primary\nlayer: database\nmax_lines_per_file: 250\nallowed_imports: [\"contracts::*\"]\nforbidden_imports: [\"client::*\"]\n---\n# 💾 Adaptateur Base de Données Primaire\n\nMigrations, modèles relationnels et persistance.\n")),
        ("data-nodes/cache-worker", format!("---\nid: cache-worker\nlayer: utility\nmax_lines_per_file: 200\nallowed_imports: [\"contracts::*\"]\nforbidden_imports: [\"client::*\"]\n---\n# ⚡ Cache & Tâches en Arrière-Plan\n\nFiles d'attente (Redis/MQ) et workers asynchrones isolables.\n")),
    ];

    let mut created = Vec::new();
    for (rel_dir, readme_content) in structure {
        let dir_path = base.join(rel_dir);
        let _ = fs::create_dir_all(&dir_path);
        let readme_path = dir_path.join("README.md");
        let _ = fs::write(&readme_path, readme_content);
        created.push(rel_dir.to_string());
    }

    // Calcule et scelle les hashes BLAKE3 de tous les README.md générés dans aether.lock.toml
    let _ = sign_project_contracts(base, None);

    // Fichier maître de directives IA optimisé AETHER_AI_RULES.md
    let rules_file = base.join("AETHER_AI_RULES.md");
    let rules_content = AuditWriter::generate_optimized_system_prompt(base, &proj_name, stack_str, 3030);
    let _ = fs::write(rules_file, rules_content);

    // Fichiers de code de démarrage avec documentation complète et style ultra-normé
    let code_files: Vec<(&str, &str)> = match stack {
        StackType::PureRust => vec![
            ("contracts/src/lib.rs", r#"//! # 📜 Contrats d'Interface & Schémas Partagés
//!
//! Contient les DTOs, les types de transfert et les ports d'abstraction.

use serde::{Deserialize, Serialize};

/// # [CONTRACT-001] Objet de transfert de données pour une réservation
///
/// ## Invariants
/// - `id` : Identifiant unique non vide
/// - `nights` : Nombre de nuitées (strictement > 0 et <= 90)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReservationDto {
    pub id: String,
    pub client_name: String,
    pub nights: u32,
    pub total_amount: u64,
}

/// # [PORT-001] Port d'accès abstrait au stockage (Découplage local/distant)
///
/// ## Spécification EARS
/// - **WHEN** l'application demande la persistance d'une réservation
/// - **THEN** l'adaptateur de données doit sauvegarder le DTO de manière atomique
pub trait RepositoryPort: Send + Sync {
    /// Sauvegarde une réservation dans le système de persistance actif
    fn save_reservation(&self, reservation: &ReservationDto) -> Result<(), String>;
    /// Récupère une réservation par son identifiant unique
    fn find_by_id(&self, id: &str) -> Result<Option<ReservationDto>, String>;
}
"#),
            ("server/domain/src/lib.rs", r#"//! # 🏛️ Couche Domaine Pur (Zéro I/O, 100% Déterministe)
//!
//! Règles métier fondamentales, calculs financiers et invariants stricts.

/// # [DOMAIN-001] Calcule le montant total d'un séjour avec application des taxes
///
/// ## Spécification EARS
/// - **WHEN** le nombre de nuits est strictement positif et inférieur ou égal à 90
/// - **THEN** le système calcule le montant brut majoré de la taxe de séjour (5%)
///
/// ## Invariants & Sécurité
/// - Fonction 100% pure, déterministe, sans effet de bord, sans I/O réseau ou disque.
/// - Complexité O(1).
///
/// ## Arguments
/// - `nights`: Nombre de nuitées (1..=90)
/// - `rate_per_night`: Tarif par nuit en centimes / wei
pub fn calculate_stay_cost(nights: u32, rate_per_night: u64) -> Result<u64, &'static str> {
    if nights == 0 {
        return Err("Le séjour doit comporter au moins 1 nuit.");
    }
    if nights > 90 {
        return Err("La durée maximale d'un séjour continu est de 90 nuits.");
    }
    let base = (nights as u64) * rate_per_night;
    let tax = base * 5 / 100;
    Ok(base + tax)
}
"#),
            ("server/application/src/lib.rs", r#"//! # ⚙️ Couche Orchestration Applicative (Cas d'Usage)
//!
//! Coordonne les flux entre le domaine pur et les adaptateurs d'infrastructure.

/// # [APP-001] Traite la création d'un dossier de réservation complet
///
/// ## Spécification EARS
/// - **WHEN** une demande utilisateur est validée par le contrat
/// - **THEN** le service applique les règles domaine et prépare l'enregistrement
pub fn process_reservation_usecase(client: &str, nights: u32, rate: u64) -> Result<u64, &'static str> {
    if client.trim().is_empty() {
        return Err("Nom de client invalide.");
    }
    let total = (nights as u64) * rate;
    Ok(total)
}
"#),
            ("server/api/src/lib.rs", r#"//! # 🌐 Entrées API & Contrôleurs RPC (Routes Réseau)
//!
//! Point de contact réseau scanné par AetherScope (Laser Orange RPC).

/// # [API-001] Point d'entrée de réservation HTTP POST /api/book
///
/// Reçoit la payload du formulaire client («Form»), la valide et l'achemine.
pub fn handle_post_book(payload: &str) -> String {
    format!("Booking route /api/book processed: {}", payload)
}
"#),
            ("client/ui/src/form.rs", r#"//! # 🎨 Formulaires et Rendu Visuel («Form»)

/// # [UI-001] Formulaire de réservation de chambre («Form»)
///
/// Stéréotype: «Form»
/// Soumet vers: `/api/book` (Connexion Laser Orange RPC)
pub fn render_booking_form(client_name: &str, nights: u32) {
    let _ = "fetch('/api/book')";
}
"#),
            ("data-nodes/db-primary/src/lib.rs", r#"//! # 💾 Adaptateur Base de Données Primaire (Repository SQLite / Postgres)

/// # [DB-001] Adaptateur de stockage primaire (SQLite local ou Postgres distant)
pub struct PrimaryDatabaseAdapter {
    pub connection_url: String,
}

impl PrimaryDatabaseAdapter {
    pub fn new(connection_url: &str) -> Self {
        Self { connection_url: connection_url.to_string() }
    }
}
"#),
        ],
        StackType::PurePythonUv => vec![
            ("contracts/schemas.py", r#""""📜 Contrats Pydantic v2 & Interfaces Partagées.

Valide de manière stricte tous les échanges entre le client, l'API et les data-nodes.
"""
from pydantic import BaseModel, Field

class BookingRequest(BaseModel):
    """[CONTRACT-001] Schéma de validation pour une demande de réservation."""
    client_name: str = Field(..., min_length=2, description="Nom complet du client")
    nights: int = Field(..., gt=0, le=90, description="Nombre de nuitées")
    rate_per_night: float = Field(default=120.0, gt=0, description="Tarif unitaire")

class BookingResponse(BaseModel):
    """[CONTRACT-002] Schéma de réponse après calcul du domaine."""
    booking_id: str
    total_amount: float
    status: str
"#),
            ("server/domain/models.py", r#""""🏛️ Couche Domaine Pur (Zéro I/O, 100% Déterministe)."""

def calculate_booking_total(nights: int, rate: float) -> float:
    """[DOMAIN-001] Calcule le montant total d'un séjour avec taxes légales.

    Spécification EARS:
        WHEN le nombre de nuitées est valide (1 à 90)
        THEN le système calcule le prix total avec 5% de taxe de séjour

    Invariants:
        Fonction 100% pure, déterministe, zéro I/O réseau ou base de données.

    Args:
        nights (int): Nombre de nuits (1-90).
        rate (float): Tarif par nuitée.

    Returns:
        float: Montant total calculé.
    """
    base = nights * rate
    return round(base * 1.05, 2)
"#),
            ("server/api/router.py", r#""""🌐 Entrées API & Contrôleurs FastAPI."""

def handle_booking_route(payload: dict) -> dict:
    """[API-001] Endpoint HTTP POST /api/book.
    
    Reçoit les entrées du formulaire client («Form») et valide via Pydantic.
    """
    return {"status": "confirmed", "route": "/api/book"}
"#),
            ("client/ui/form.py", r#""""🎨 Formulaire Client («Form»)."""

def render_booking_form():
    """[UI-001] Composant Formulaire client.
    Soumet vers: /api/book (Laser Orange RPC)
    """
    url = "/api/book"
    return f"fetch('{url}')"
"#),
        ],
        StackType::PureNodePnpm => vec![
            ("contracts/index.ts", r#"import { z } from 'zod';

/**
 * [CONTRACT-001] Schéma Zod de validation de réservation hôtelière
 */
export const BookingSchema = z.object({
  clientName: z.string().min(2),
  nights: z.number().int().positive().max(90),
  ratePerNight: z.number().positive().default(120),
});

export type BookingInput = z.infer<typeof BookingSchema>;
"#),
            ("server/domain/pricing.ts", r#"/**
 * [DOMAIN-001] Calcule le coût global d'une réservation.
 *
 * @remarks
 * Spécification EARS:
 * - WHEN les entrées sont validées par BookingSchema
 * - THEN le domaine applique la taxe de séjour de 5%
 *
 * @param nights - Nombre de nuitées strictement positif
 * @param rate - Tarif unitaire unifié
 * @returns Montant total calculé
 */
export function computeStayTotal(nights: number, rate: number): number {
  const base = nights * rate;
  return Math.round(base * 1.05 * 100) / 100;
}
"#),
            ("server/api/routes.ts", r#"/**
 * [API-001] Handler HTTP POST /api/book
 * Route déclarée reliant le formulaire client («Form»).
 */
export async function handleBookingApi(req: any) {
  return { status: 'confirmed', route: '/api/book' };
}
"#),
            ("client/ui/BookingForm.tsx", r#"/**
 * [UI-001] Formulaire de réservation («Form»)
 * Cible: fetch('/api/book') -> Laser Orange RPC
 */
export function BookingForm() {
  const submit = () => fetch('/api/book');
  return null;
}
"#),
        ],
        StackType::HybridMmoWeb3 => vec![
            ("contracts/packets.rs", r#"//! # 📜 Définitions des Paquets WebSocket & ABIs Web3

/// # [WS-001] Paquet émis par le client joueur
#[derive(Debug, Clone)]
pub struct ClientPacketMove {
    pub player_id: String,
    pub x: f32,
    pub y: f32,
}

/// # [WS-002] Paquet d'état synchronisé par le serveur
#[derive(Debug, Clone)]
pub struct ServerPacketStateUpdate {
    pub tick: u64,
    pub entities_count: usize,
}
"#),
            ("server/domain/world.rs", r#"//! # 🏛️ Moteur de Monde & Simulation Stellaire MMO (Pur)

/// # [DOMAIN-001] Met à jour la position spatiale d'une entité
///
/// ## Spécification EARS
/// - **WHEN** un paquet de mouvement arrive dans le tick actif
/// - **THEN** le système valide les collisions et met à jour les coordonnées
pub fn update_player_coord(current: f32, delta: f32) -> f32 {
    current + delta
}
"#),
            ("server/api/websocket.rs", r#"//! # ⚡ Contrôleur WebSocket & Dispatch de Paquets

/// # [WS-ROUTE-001] Router de paquets WebSocket temps réel
pub fn handle_client_packet(packet: &str) -> String {
    format!("ServerPacket::StateUpdate for {}", packet)
}
"#),
            ("client/ui/GameCanvas.ts", r#"/**
 * [UI-001] Rendu Canvas 2D/WebGL PixiJS («Form» & Realtime)
 * Émet vers: socket.emit('ClientPacket::Move')
 */
export class GameCanvas {
  sendMove(x: number, y: number) {
    // socket.emit('ClientPacket::Move')
  }
}
"#),
        ],
    };

    for (rel_file, file_content) in code_files {
        let fpath = base.join(rel_file);
        if let Some(parent) = fpath.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&fpath, file_content);
    }

    created
}

pub fn pick_folder_dialog(title: &str) -> Option<PathBuf> {
    // 1. Essai avec rfd
    if let Some(folder) = rfd::FileDialog::new().set_title(title).pick_folder() {
        return Some(folder);
    }

    // 2. Fallback garanti sur Windows via System.Windows.Forms
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let ps_cmd = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = '{}'; $f.ShowNewFolderButton = $true; if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ Write-Output $f.SelectedPath }}"#,
            title
        );
        let mut cmd = std::process::Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", &ps_cmd]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() && Path::new(&path_str).is_dir() {
                    return Some(PathBuf::from(path_str));
                }
            }
        }
    }
    None
}
