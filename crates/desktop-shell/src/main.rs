#![windows_subsystem = "windows"]

mod api_routes;
mod local_guard;
mod mcp;
mod viewer;

use api_routes::*;
use ai_file_bridge::{AuditWriter, FileWatcherBridge, HandshakePacker};
use ast_scanner::ProjectScanner;
use clap::Parser;
use security_linter::contract::ReadmeContract;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use tao::{
    dpi::LogicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "AetherScope V2 - Autonomous Graph Engine & AI Guardrail Bridge", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = ".")]
    pub path: String,

    #[arg(long, default_value_t = 3030)]
    pub port: u16,

    #[arg(long, default_value_t = false)]
    pub audit_only: bool,

    #[arg(long, default_value_t = false)]
    pub no_gui: bool,

    #[arg(long, default_value_t = false)]
    pub mcp: bool,
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let _ = std::fs::write("desktop-shell-panic.log", format!("PANIC: {:?}", info));
    }));
    if let Err(e) = real_main() {
        let _ = std::fs::write("desktop-shell-error.log", format!("MAIN ERROR: {:?}", e));
    }
}

fn real_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let port = args.port;
    let audit_only = args.audit_only;
    let no_gui = args.no_gui;
    let mcp = args.mcp;

    if mcp {
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(mcp::run_mcp_server(args));
    }

    if audit_only {
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(run_audit_cli(args));
    }

    // Démarre le serveur backend Axum et le File-Watcher dans un thread d'arrière-plan Tokio
    let server_args = args.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Échec initialisation Tokio runtime");
        rt.block_on(async {
            if let Err(e) = run_server_and_watcher(server_args).await {
                let _ = std::fs::write("server-error.log", format!("SERVER ERROR: {:?}", e));
                eprintln!("⚠️ Erreur serveur interne : {}", e);
            }
        });
    });

    if no_gui {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    // =========================================================================
    // NATIVE DESKTOP WINDOW (FULL IN-EXE GUI VIA TAO + WRY / TAURI ENGINE)
    // =========================================================================
    std::thread::sleep(std::time::Duration::from_millis(350));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("🪐 AetherScope V2 — Architectural Intelligence & Live GPU Cockpit")
        .with_maximized(true)
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(LogicalSize::new(800.0, 500.0))
        .build(&event_loop)?;

    let app_url = format!("http://127.0.0.1:{}", port);
    let _webview = WebViewBuilder::new(&window)
        .with_url(&app_url)
        .with_devtools(true)
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}

fn extract_live_audit_from_graph(
    graph: &core_graph::graph::AetherGraph,
) -> (Vec<security_linter::linter::LinterViolation>, usize) {
    let mut violations = Vec::new();
    let mut file_count = 0usize;
    for node in &graph.nodes {
        if node.kind == core_graph::bubble::SymbolKind::File {
            file_count += 1;
        }
        for rec in &node.metadata.cve_anomalies {
            violations.push(security_linter::linter::LinterViolation {
                file_path: rec.file_path.clone(),
                line_number: rec.line_number,
                rule_id: rec.rule_id.clone(),
                cve_id: rec.cve_id.clone(),
                severity: rec.severity.clone(),
                message: rec.message.clone(),
                transmitted_data: rec.transmitted_data.clone(),
                transmitted_type: rec.transmitted_type.clone(),
                remediation: rec.remediation.clone(),
            });
        }
    }
    (violations, file_count)
}

async fn run_audit_cli(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = fs::canonicalize(Path::new(&args.path)).unwrap_or_else(|_| PathBuf::from(&args.path));
    AuditWriter::ensure_master_rules_file(&project_root)?;
    let scanner = ProjectScanner::default();
    let graph = scanner.scan_project(&project_root);

    let (all_violations, file_count) = extract_live_audit_from_graph(&graph);
    AuditWriter::write_live_audit(&project_root, &all_violations, file_count)?;
    Ok(())
}

pub async fn run_server_and_watcher(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = fs::canonicalize(Path::new(&args.path)).unwrap_or_else(|_| PathBuf::from(&args.path));

    AuditWriter::ensure_master_rules_file(&project_root)?;

    let scanner = ProjectScanner::default();
    let graph = scanner.scan_project(&project_root);

    let (all_violations, audited_files_count) = extract_live_audit_from_graph(&graph);
    let _ = AuditWriter::write_live_audit(&project_root, &all_violations, audited_files_count);

    let shared_state = Arc::new(AppState {
        graph: RwLock::new(graph),
        project_root: RwLock::new(project_root.clone()),
        telemetry: RwLock::new(None),
        chat_messages: RwLock::new(Vec::new()),
        graph_version: std::sync::atomic::AtomicU64::new(1),
        custom_exclusions: RwLock::new(Vec::new()),
        active_cve_rules: RwLock::new(None),
    });

    // FileWatcher background loop
    let watcher_root = project_root.clone();
    let watcher_state = shared_state.clone();

    tokio::spawn(async move {
        if let Ok(mut watcher) = FileWatcherBridge::new(&watcher_root) {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                let current_root = watcher_state.project_root.read().await.clone();
                if current_root != watcher.project_root {
                    let _ = watcher.switch_root(&current_root);
                }

                let changed = watcher.poll_changed_files();
                if !changed.is_empty() {
                    let custom_excl = watcher_state.custom_exclusions.read().await.clone();
                    let scan_root = current_root.clone();
                    if let Ok(updated_graph) = tokio::task::spawn_blocking(move || {
                        let scanner = ProjectScanner::default();
                        scanner.scan_project_with_exclusions(&scan_root, &custom_excl)
                    })
                    .await
                    {
                        let (violations, file_count) = extract_live_audit_from_graph(&updated_graph);
                        let _ = AuditWriter::write_live_audit(&current_root, &violations, file_count);
                        let mut graph_guard = watcher_state.graph.write().await;
                        *graph_guard = updated_graph;
                        watcher_state.graph_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
        }
    });

    let state_viewer = shared_state.clone();
    let state_graph = shared_state.clone();
    let state_graph_version = shared_state.clone();
    let state_projects = shared_state.clone();
    let state_analyze = shared_state.clone();
    let state_analyze_get = shared_state.clone();
    let state_export = shared_state.clone();
    let state_savedoc = shared_state.clone();
    let state_sign_contracts = shared_state.clone();
    let state_pick_ws = shared_state.clone();
    let state_pick_proj = shared_state.clone();
    let state_init_proj = shared_state.clone();
    let state_workspace = shared_state.clone();
    let state_telemetry = shared_state.clone();
    let state_ai_target = shared_state.clone();
    let state_chat = shared_state.clone();
    let state_create_file = shared_state.clone();
    let state_sysprompt = shared_state.clone();
    let state_file_content = shared_state.clone();
    let state_ai_cancel = shared_state.clone();
    let state_ai_session = shared_state.clone();
    let state_ai_cli = shared_state.clone();
    let state_settings = shared_state.clone();
    let state_tasks = shared_state.clone();
    let server_port = args.port;

    let app = axum::Router::new()
        .route(
            "/",
            axum::routing::get(move || {
                let state = state_viewer.clone();
                async move {
                    let g = state.graph.read().await;
                    let html = viewer::generate_standalone_html(&g);
                    (
                        [(axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
                        axum::response::Html(html),
                    )
                }
            }),
        )
        .route(
            "/api/projects",
            axum::routing::get(move || {
                let state = state_projects.clone();
                async move {
                    let p_root = state.project_root.read().await;
                    let projects = discover_candidate_projects(&p_root);
                    let default_project = Some(clean_path_string(&p_root));
                    axum::Json(ProjectsResponse {
                        projects,
                        default_project,
                    })
                }
            }),
        )
        .route(
            "/api/workspace",
            axum::routing::get({
                let state = state_workspace.clone();
                move || async move {
                    let p = state.project_root.read().await;
                    axum::Json(serde_json::json!({ "workspace": p.to_string_lossy() }))
                }
            })
            .post({
                let state = state_workspace.clone();
                move |axum::extract::Json(payload): axum::extract::Json<serde_json::Value>| async move {
                    if let Some(ws) = payload.get("workspace").and_then(|v| v.as_str()) {
                        let mut p = state.project_root.write().await;
                        *p = PathBuf::from(ws);
                    }
                    axum::Json(serde_json::json!({ "status": "ok" }))
                }
            }),
        )
        // Dialogue natif : Choisir le dossier contenant TOUS les projets
        .route(
            "/api/dialog/pick_workspace",
            axum::routing::post(move || {
                let state = state_pick_ws.clone();
                async move {
                    let chosen = tokio::task::spawn_blocking(|| {
                        pick_folder_dialog("Sélectionnez le dossier racine contenant tous les projets")
                    })
                    .await
                    .ok()
                    .flatten();

                    if let Some(folder) = chosen {
                        let path_str = folder.to_string_lossy().to_string();
                        let projects = discover_candidate_projects(&folder);
                        let mut p_guard = state.project_root.write().await;
                        *p_guard = folder;
                        axum::Json(PickWorkspaceResponse {
                            status: "ok".to_string(),
                            workspace: Some(path_str),
                            projects,
                        })
                    } else {
                        axum::Json(PickWorkspaceResponse {
                            status: "cancelled".to_string(),
                            workspace: None,
                            projects: Vec::new(),
                        })
                    }
                }
            }),
        )
        // Dialogue natif : Choisir 1 projet unique
        .route(
            "/api/dialog/pick_project",
            axum::routing::post(move || {
                let state = state_pick_proj.clone();
                async move {
                    let chosen = tokio::task::spawn_blocking(|| {
                        pick_folder_dialog("Sélectionnez le projet à ouvrir et analyser")
                    })
                    .await
                    .ok()
                    .flatten();

                    if let Some(folder) = chosen {
                        let path_str = folder.to_string_lossy().to_string();
                        let custom_excl = state.custom_exclusions.read().await.clone();
                        let scan_folder = folder.clone();
                        let new_graph = tokio::task::spawn_blocking(move || {
                            let scanner = ProjectScanner::default();
                            scanner.scan_project_with_exclusions(&scan_folder, &custom_excl)
                        })
                        .await
                        .unwrap_or_else(|_| core_graph::graph::AetherGraph::new("Project"));

                        let _ = AuditWriter::ensure_master_rules_file(&folder);
                        let (violations, file_count) = extract_live_audit_from_graph(&new_graph);
                        let _ = AuditWriter::write_live_audit(&folder, &violations, file_count);

                        let mut g_guard = state.graph.write().await;
                        *g_guard = new_graph.clone();
                        let mut p_guard = state.project_root.write().await;
                        *p_guard = folder;
                        state.graph_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        axum::Json(PickProjectResponse {
                            status: "ok".to_string(),
                            project_path: Some(path_str),
                            graph: Some(new_graph),
                        })
                    } else {
                        axum::Json(PickProjectResponse {
                            status: "cancelled".to_string(),
                            project_path: None,
                            graph: None,
                        })
                    }
                }
            }),
        )
        // Scaffolding d'architecture projet
        .route(
            "/api/init_project",
            axum::routing::post(move |axum::extract::Json(req): axum::extract::Json<InitProjectRequest>| {
                let _state = state_init_proj.clone();
                async move {
                    let base = Path::new(&req.path);
                    let created = init_project_structure(base, &req.stack);
                    axum::Json(InitProjectResponse {
                        status: "ok".to_string(),
                        message: format!("Projet initialisé avec {} dossiers et contrats README.md signés BLAKE3", created.len()),
                        created_dirs: created,
                    })
                }
            }),
        )
        .route(
            "/api/analyze",
            axum::routing::post({
                let state = state_analyze.clone();
                move |axum::extract::Json(req): axum::extract::Json<AnalyzeRequest>| {
                    let state = state.clone();
                    async move {
                        let raw = req.path.trim().trim_matches('"').trim_matches('\'');
                        let clean = raw.strip_prefix(r"\\?\").unwrap_or(raw);
                        let path = fs::canonicalize(&clean).unwrap_or_else(|_| PathBuf::from(&clean));
                        let mut custom_excl = state.custom_exclusions.read().await.clone();
                        if let Some(req_excl) = req.exclusions {
                            custom_excl = req_excl.clone();
                            let mut excl_guard = state.custom_exclusions.write().await;
                            *excl_guard = req_excl;
                        }
                        let scan_path = path.clone();
                        let new_graph = tokio::task::spawn_blocking(move || {
                            let scanner = ProjectScanner::default();
                            scanner.scan_project_with_exclusions(&scan_path, &custom_excl)
                        })
                        .await
                        .unwrap_or_else(|_| core_graph::graph::AetherGraph::new("Project"));

                        let _ = AuditWriter::ensure_master_rules_file(&path);
                        let (violations, file_count) = extract_live_audit_from_graph(&new_graph);
                        let _ = AuditWriter::write_live_audit(&path, &violations, file_count);

                        let mut g_guard = state.graph.write().await;
                        *g_guard = new_graph.clone();
                        let mut p_guard = state.project_root.write().await;
                        *p_guard = path;
                        state.graph_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        axum::Json(new_graph)
                    }
                }
            })
            .get({
                let state = state_analyze_get.clone();
                move |axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>| {
                    let state = state.clone();
                    async move {
                        let p_str = query.get("path").cloned().unwrap_or_default();
                        let raw = p_str.trim().trim_matches('"').trim_matches('\'');
                        let clean = raw.strip_prefix(r"\\?\").unwrap_or(raw);
                        let path = fs::canonicalize(&clean).unwrap_or_else(|_| PathBuf::from(&clean));
                        let custom_excl = state.custom_exclusions.read().await.clone();
                        let scan_path = path.clone();
                        let new_graph = tokio::task::spawn_blocking(move || {
                            let scanner = ProjectScanner::default();
                            scanner.scan_project_with_exclusions(&scan_path, &custom_excl)
                        })
                        .await
                        .unwrap_or_else(|_| core_graph::graph::AetherGraph::new("Project"));

                        let _ = AuditWriter::ensure_master_rules_file(&path);
                        let (violations, file_count) = extract_live_audit_from_graph(&new_graph);
                        let _ = AuditWriter::write_live_audit(&path, &violations, file_count);

                        let mut g_guard = state.graph.write().await;
                        *g_guard = new_graph.clone();
                        let mut p_guard = state.project_root.write().await;
                        *p_guard = path;
                        state.graph_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        axum::Json(new_graph)
                    }
                }
            }),
        )
        .route(
            "/api/graph",
            axum::routing::get(move || {
                let state = state_graph.clone();
                async move {
                    let g = state.graph.read().await;
                    axum::Json(g.clone())
                }
            }),
        )
        .route(
            "/api/graph/version",
            axum::routing::get(move || {
                let state = state_graph_version.clone();
                async move {
                    let v = state.graph_version.load(std::sync::atomic::Ordering::SeqCst);
                    axum::Json(serde_json::json!({ "version": v }))
                }
            }),
        )
        .route(
            "/api/export",
            axum::routing::post(move |axum::extract::Json(req): axum::extract::Json<ExportRequest>| {
                let state = state_export.clone();
                async move {
                    let g = state.graph.read().await;
                    let p_root = state.project_root.read().await;
                    let target_dir = req.path.map(PathBuf::from).unwrap_or_else(|| p_root.clone());
                    let out_path = target_dir.join("aetherscope_viewer.html");
                    let html = viewer::generate_standalone_html(&g);
                    let _ = fs::write(&out_path, html);
                    axum::Json(ExportResponse {
                        status: "ok".to_string(),
                        viewer_path: out_path.to_string_lossy().to_string(),
                        total_nodes: g.nodes.len(),
                        total_edges: g.edges.len(),
                    })
                }
            }),
        )
        .route(
            "/api/save_doc",
            axum::routing::post(move |axum::extract::Json(req): axum::extract::Json<SaveDocRequest>| {
                let state = state_savedoc.clone();
                async move {
                    let base = Path::new(&req.project_path);
                    let path = if req.folder.is_empty() || req.folder == "root" || req.folder == "." {
                        base.join(&req.file_name)
                    } else {
                        base.join(&req.folder).join(&req.file_name)
                    };
                    if let Some(parent) = path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = fs::write(&path, &req.content);

                    let mut blake3_hash = None;
                    if req.file_name.eq_ignore_ascii_case("README.md") {
                        if let Ok((_, h)) = sign_project_contracts(base, Some(&req.folder)) {
                            blake3_hash = h;
                        }
                        let scanner = ProjectScanner::default();
                        let updated_graph = scanner.scan_project(base);
                        let mut g_guard = state.graph.write().await;
                        *g_guard = updated_graph;
                        state.graph_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }

                    axum::Json(SaveDocResponse {
                        status: "ok".to_string(),
                        saved_path: path.to_string_lossy().to_string(),
                        blake3_hash,
                    })
                }
            }),
        )
        // Signature officielle des contrats BLAKE3 dans aether.lock.toml
        .route(
            "/api/contracts/sign",
            axum::routing::post(move |axum::extract::Json(req): axum::extract::Json<SignContractsRequest>| {
                let state = state_sign_contracts.clone();
                async move {
                    let p_root = state.project_root.read().await.clone();
                    let target_root = req
                        .project_path
                        .as_deref()
                        .filter(|s| !s.trim().is_empty())
                        .map(PathBuf::from)
                        .unwrap_or(p_root);

                    let (signed_count, blake3_hash) = sign_project_contracts(&target_root, req.folder.as_deref())
                        .unwrap_or((0, None));

                    let scanner = ProjectScanner::default();
                    let new_graph = scanner.scan_project(&target_root);
                    {
                        let mut g_guard = state.graph.write().await;
                        *g_guard = new_graph.clone();
                    }
                    state.graph_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                    axum::Json(SignContractsResponse {
                        status: "ok".to_string(),
                        message: format!("{} contrat(s) README.md signé(s) avec BLAKE3 dans aether.lock.toml", signed_count),
                        signed_count,
                        blake3_hash,
                        graph: new_graph,
                    })
                }
            }),
        )
        // Pont IA Zero-Shell : écriture de .aether/ACTIVE_TARGET.md et génération du paquet <aether_handshake version="2.0">
        .route(
            "/api/ai/target",
            axum::routing::post(move |axum::extract::Json(req): axum::extract::Json<AiTargetRequest>| {
                let state = state_ai_target.clone();
                async move {
                    let p_root = state.project_root.read().await.clone();
                    let g = state.graph.read().await;

                    let sym_opt = g
                        .nodes
                        .iter()
                        .find(|n| {
                            n.id == req.symbol_id
                                || n.name == req.symbol_id
                                || n.file_path == req.symbol_id
                                || n.id.ends_with(&format!("::{}", req.symbol_id))
                        })
                        .cloned();

                    if let Some(sym) = sym_opt {
                        let incoming_edges: Vec<&core_graph::edge::AffiliationEdge> = g
                            .edges
                            .iter()
                            .filter(|e| e.target == sym.id && e.relation != "contains")
                            .collect();
                        let outgoing_edges: Vec<&core_graph::edge::AffiliationEdge> = g
                            .edges
                            .iter()
                            .filter(|e| e.source == sym.id && e.relation != "contains")
                            .collect();

                        let categorized_in = core_graph::edge::CategorizedLiaisons::from_edges(&incoming_edges, &sym.id, true);
                        let categorized_out = core_graph::edge::CategorizedLiaisons::from_edges(&outgoing_edges, &sym.id, false);

                        let callers: Vec<String> = incoming_edges.iter().map(|e| e.source.clone()).collect();
                        let dependencies: Vec<String> = outgoing_edges.iter().map(|e| e.target.clone()).collect();

                        // Recherche du contrat README.md applicable au dossier du symbole
                        let readme_path = if sym.folder.is_empty() || sym.folder == "root" {
                            p_root.join("README.md")
                        } else {
                            let direct = p_root.join(&sym.folder).join("README.md");
                            if direct.exists() {
                                direct
                            } else {
                                p_root.join(&sym.galaxy).join("README.md")
                            }
                        };

                        let contract = fs::read_to_string(&readme_path)
                            .ok()
                            .and_then(|c| ReadmeContract::parse(&sym.folder, &c).ok());

                        let _ = AuditWriter::write_active_target_categorized(
                            &p_root,
                            &sym,
                            &categorized_in,
                            &categorized_out,
                            contract.as_ref(),
                        );

                        let action_type = req.action_type.as_deref().unwrap_or("Target");
                        let handshake_xml = HandshakePacker::build_categorized_prompt(
                            action_type,
                            &sym,
                            &categorized_in,
                            &categorized_out,
                            contract.as_ref(),
                        );

                        let active_target_path = p_root.join(".aether").join("ACTIVE_TARGET.md").to_string_lossy().to_string();

                        let mut doc_content = if !sym.docstring.is_empty() {
                            sym.docstring.clone()
                        } else if let Some(raw) = &sym.metadata.raw_content {
                            raw.clone()
                        } else {
                            String::new()
                        };

                        // Fallback lecture de commentaires sources au-dessus de la ligne du symbole
                        if doc_content.is_empty() && !sym.file_path.is_empty() && sym.line_number > 0 {
                            let src_path = p_root.join(&sym.file_path);
                            if let Ok(file_str) = fs::read_to_string(&src_path) {
                                let lines: Vec<&str> = file_str.lines().collect();
                                let target_idx = sym.line_number.saturating_sub(1);
                                let mut collected = Vec::new();
                                for i in (0..target_idx).rev().take(30) {
                                    if let Some(line) = lines.get(i) {
                                        let trimmed = line.trim();
                                        if trimmed.is_empty() {
                                            if collected.is_empty() { continue; } else { break; }
                                        }
                                        if trimmed.starts_with("#[") || trimmed.starts_with("#![") || trimmed == "]" || (trimmed.starts_with('@') && !trimmed.starts_with("@param") && !trimmed.starts_with("@return")) {
                                            continue;
                                        }
                                        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                                            collected.push(trimmed.trim_start_matches('/').trim().to_string());
                                        } else if trimmed.starts_with("//") {
                                            collected.push(trimmed.trim_start_matches('/').trim().to_string());
                                        } else if trimmed.starts_with('#') {
                                            collected.push(trimmed.trim_start_matches('#').trim().to_string());
                                        } else if trimmed.ends_with("*/") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                                            collected.push(trimmed.trim_matches(|c| c == '/' || c == '*').trim().to_string());
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                if !collected.is_empty() {
                                    collected.reverse();
                                    doc_content = collected.join("\n");
                                }
                            }
                        }

                        let doc_display = if doc_content.is_empty() {
                            "*(Aucun commentaire AST direct détecté)*".to_string()
                        } else {
                            doc_content.clone()
                        };

                        let callers_md = categorized_in.to_compact_markdown("amont", 4);
                        let deps_md = categorized_out.to_compact_markdown("aval", 4);

                        let formatted_doc_and_liaisons = format!(
                            "# 🎯 Cible : `{}`\n- **Fichier :** `{}:{}`\n- **Architecture :** Galaxie `{}` | Couche `{}` | Stéréotype `{}`\n\n## 📖 DOCUMENTATION & SPÉCIFICATIONS AST :\n{}\n\n## 📥 FLUX AMONT / APPELANTS (Total: {}):\n{}\n## 📤 FLUX AVAL / DÉPENDANCES & LIAISONS (Total: {}):\n{}\n## 🔒 CONTRAT DU MODULE (`{}`):\n- Imports autorisés : `{:?}`\n- Imports INTERDITS : `{:?}`\n- Lignes max : {}\n- Exigences EARS : {}",
                            sym.name,
                            sym.file_path,
                            sym.line_number,
                            sym.galaxy,
                            sym.layer,
                            sym.stereotype,
                            doc_display,
                            categorized_in.total(),
                            callers_md,
                            categorized_out.total(),
                            deps_md,
                            sym.folder,
                            contract.as_ref().map(|c| &c.frontmatter.allowed_imports).unwrap_or(&vec![]),
                            contract.as_ref().map(|c| &c.frontmatter.forbidden_imports).unwrap_or(&vec![]),
                            contract.as_ref().map(|c| c.frontmatter.max_lines_per_file).unwrap_or(300),
                            contract.as_ref().map(|c| c.ears_requirements.len()).unwrap_or(sym.metadata.ears_count)
                        );

                        axum::Json(AiTargetResponse {
                            status: "ok".to_string(),
                            active_target_path,
                            handshake_xml,
                            blake3_hash: sym.metadata.blake3_hash.or_else(|| contract.as_ref().map(|c| c.blake3_hash.clone())),
                            lock_verified: sym.metadata.lock_verified,
                            ears_count: contract.as_ref().map(|c| c.ears_requirements.len()).unwrap_or(sym.metadata.ears_count),
                            symbol_name: Some(sym.name.clone()),
                            file_path: Some(format!("{}:{}", sym.file_path, sym.line_number)),
                            layer: Some(sym.layer.clone()),
                            galaxy: Some(sym.galaxy.clone()),
                            docstring: if doc_content.is_empty() { None } else { Some(doc_content) },
                            callers,
                            dependencies,
                            categorized_incoming: Some(categorized_in),
                            categorized_outgoing: Some(categorized_out),
                            formatted_doc_and_liaisons: Some(formatted_doc_and_liaisons),
                        })
                    } else {
                        axum::Json(AiTargetResponse {
                            status: "not_found".to_string(),
                            active_target_path: String::new(),
                            handshake_xml: String::new(),
                            blake3_hash: None,
                            lock_verified: None,
                            ears_count: 0,
                            symbol_name: None,
                            file_path: None,
                            layer: None,
                            galaxy: None,
                            docstring: None,
                            callers: vec![],
                            dependencies: vec![],
                            categorized_incoming: None,
                            categorized_outgoing: None,
                            formatted_doc_and_liaisons: None,
                        })
                    }
                }
            }),
        )
        // Télémetrie IA
        .route(
            "/api/ai/telemetry",
            axum::routing::get({
                let state = state_telemetry.clone();
                move || async move {
                    let t = state.telemetry.read().await;
                    axum::Json(serde_json::json!({ "telemetry": *t }))
                }
            })
            .post({
                let state = state_telemetry.clone();
                move |axum::extract::Json(mut event): axum::extract::Json<AiTelemetryEvent>| async move {
                    if event.timestamp == 0 {
                        event.timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                    }
                    let mut t = state.telemetry.write().await;
                    *t = Some(event);
                    axum::Json(serde_json::json!({ "status": "ok" }))
                }
            }),
        )
        // Chat IA local
        .route(
            "/api/ai/chat",
            axum::routing::get({
                let state = state_chat.clone();
                move || async move {
                    let msgs = state.chat_messages.read().await;
                    let last_is_user = msgs.last().map(|m| m.sender == "user").unwrap_or(false);
                    let agent_status = if last_is_user { "thinking" } else { "ready" };
                    axum::Json(ChatMessagesResponse {
                        messages: msgs.clone(),
                        agent_status: agent_status.to_string(),
                    })
                }
            })
            .post({
                let state = state_chat.clone();
                move |axum::extract::Json(req): axum::extract::Json<SendChatMessageRequest>| async move {
                    let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
                    let msg = AiChatMessage {
                        id: format!("msg_{}", now_ms),
                        sender: req.sender.unwrap_or_else(|| "user".to_string()),
                        text: req.text,
                        target_symbol: req.target_symbol,
                        timestamp: now_ms,
                    };
                    let mut msgs = state.chat_messages.write().await;
                    msgs.push(msg.clone());
                    axum::Json(serde_json::json!({ "status": "ok", "message": msg }))
                }
            }),
        )
        .route(
            "/api/ai/chat/clear",
            axum::routing::post({
                let state = state_chat.clone();
                move || async move {
                    let mut msgs = state.chat_messages.write().await;
                    msgs.clear();
                    axum::Json(serde_json::json!({ "status": "ok" }))
                }
            }),
        )
        .route(
            "/api/ai/status",
            axum::routing::get(|| async { axum::Json(serde_json::json!({ "status": "idle" })) }),
        )
        .route(
            "/api/ai/cancel",
            axum::routing::post({
                let state = state_ai_cancel.clone();
                move || async move {
                    let mut t = state.telemetry.write().await;
                    *t = None;
                    let mut msgs = state.chat_messages.write().await;
                    if let Some(last) = msgs.last() {
                        if last.sender == "user" {
                            let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
                            msgs.push(AiChatMessage {
                                id: format!("msg_{}", now_ms),
                                sender: "ai".to_string(),
                                text: "⚠️ Requête interrompue par l'utilisateur.".to_string(),
                                target_symbol: None,
                                timestamp: now_ms,
                            });
                        }
                    }
                    axum::Json(serde_json::json!({ "status": "ok", "message": "Requête IA annulée" }))
                }
            }),
        )
        .route(
            "/api/ai/new_session",
            axum::routing::post({
                let state = state_ai_session.clone();
                move || async move {
                    let mut msgs = state.chat_messages.write().await;
                    msgs.clear();
                    let mut t = state.telemetry.write().await;
                    *t = None;
                    axum::Json(serde_json::json!({ "status": "ok", "message": "Nouvelle session démarrée" }))
                }
            }),
        )
        .route(
            "/api/ai/launch_cli",
            axum::routing::post({
                let _state = state_ai_cli.clone();
                move |axum::extract::Json(req): axum::extract::Json<LaunchCliRequest>| async move {
                    let tool = req.tool.unwrap_or_else(|| "gemini".to_string());
                    axum::Json(serde_json::json!({
                        "status": "ok",
                        "message": format!("Terminal CLI {} initialisé", tool),
                        "tool": tool,
                        "project_path": req.project_path,
                        "prompt": req.prompt
                    }))
                }
            }),
        )
        .route(
            "/api/ai/system_prompt",
            axum::routing::get({
                let state = state_sysprompt.clone();
                move || async move {
                    let p_root = state.project_root.read().await;
                    let proj_name = p_root.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Project".to_string());
                    
                    let stack_str = AuditWriter::detect_project_stack(&p_root);
                    let prompt = AuditWriter::generate_optimized_system_prompt(&p_root, &proj_name, stack_str, server_port);
                    axum::Json(serde_json::json!({
                        "status": "ok",
                        "project_name": proj_name,
                        "project_path": p_root.to_string_lossy(),
                        "stack": stack_str,
                        "system_prompt": prompt
                    }))
                }
            }),
        )
        // Gestionnaire de Tâches & Quêtes (Model & User)
        .route(
            "/api/tasks",
            axum::routing::get({
                let state = state_tasks.clone();
                move || async move {
                    let p_root = state.project_root.read().await;
                    let tasks = ai_file_bridge::TaskManager::list_tasks(&p_root);
                    axum::Json(serde_json::json!({ "status": "ok", "tasks": tasks }))
                }
            })
            .post({
                let state = state_tasks.clone();
                move |axum::extract::Json(req): axum::extract::Json<CreateTaskApiRequest>| async move {
                    let p_root = state.project_root.read().await;
                    let kind = req.kind.as_deref().map(ai_file_bridge::TaskKind::from_str_loose).unwrap_or_default();
                    let author = req.author.as_deref().unwrap_or("user");
                    match ai_file_bridge::TaskManager::create_task(
                        &p_root,
                        &req.title,
                        kind,
                        req.target_symbol.as_deref(),
                        req.target_file.as_deref(),
                        author,
                        req.priority.as_deref(),
                        req.initial_readme.as_deref(),
                    ) {
                        Ok(task) => axum::Json(serde_json::json!({ "status": "ok", "task": task })),
                        Err(e) => axum::Json(serde_json::json!({ "status": "error", "message": format!("{}", e) })),
                    }
                }
            }),
        )
        .route(
            "/api/tasks/update",
            axum::routing::post({
                let state = state_tasks.clone();
                move |axum::extract::Json(req): axum::extract::Json<UpdateTaskApiRequest>| async move {
                    let p_root = state.project_root.read().await;
                    let status = req.status.as_deref().map(ai_file_bridge::TaskStatus::from_str_loose);
                    match ai_file_bridge::TaskManager::update_task(
                        &p_root,
                        &req.task_id,
                        status,
                        req.progress,
                        req.progress_note.as_deref(),
                        req.full_readme.as_deref(),
                    ) {
                        Ok(Some(task)) => axum::Json(serde_json::json!({ "status": "ok", "task": task })),
                        Ok(None) => axum::Json(serde_json::json!({ "status": "not_found", "message": "Tâche introuvable" })),
                        Err(e) => axum::Json(serde_json::json!({ "status": "error", "message": format!("{}", e) })),
                    }
                }
            }),
        )
        .route(
            "/api/tasks/delete",
            axum::routing::post({
                let state = state_tasks.clone();
                move |axum::extract::Json(req): axum::extract::Json<DeleteTaskApiRequest>| async move {
                    let p_root = state.project_root.read().await;
                    match ai_file_bridge::TaskManager::delete_task(&p_root, &req.task_id) {
                        Ok(deleted) => axum::Json(serde_json::json!({ "status": "ok", "deleted": deleted })),
                        Err(e) => axum::Json(serde_json::json!({ "status": "error", "message": format!("{}", e) })),
                    }
                }
            }),
        )
        // Création de fichier assistée par l'Inspecteur / IA
        .route(
            "/api/ai/create_file",
            axum::routing::post(move |axum::extract::Json(req): axum::extract::Json<CreateFileRequest>| {
                let state = state_create_file.clone();
                async move {
                    let target_path = Path::new(&req.project_path).join(&req.file_path);
                    if let Some(parent) = target_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = fs::write(&target_path, &req.content);

                    let scanner = ProjectScanner::default();
                    let new_graph = scanner.scan_project(&req.project_path);
                    let mut g_guard = state.graph.write().await;
                    *g_guard = new_graph.clone();
                    state.graph_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                    let created_symbol = req.file_path.replace(['/', '\\'], "::");
                    axum::Json(CreateFileResponse {
                        status: "ok".to_string(),
                        file_path: req.file_path,
                        created_symbol,
                        graph: new_graph.clone(),
                        data: new_graph,
                    })
                }
            }),
        )
        // Chargement à la demande de fichier / README (Lazy-loading pour booster les perfs)
        .route(
            "/api/file_content",
            axum::routing::get({
                let state = state_file_content.clone();
                move |axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>| {
                    let state = state.clone();
                    async move {
                        let rel_path = query.get("path").cloned().unwrap_or_default();
                        let p_root = state.project_root.read().await;
                        let full_path = if Path::new(&rel_path).is_absolute() {
                            PathBuf::from(&rel_path)
                        } else {
                            p_root.join(&rel_path)
                        };
                        match fs::read_to_string(&full_path) {
                            Ok(content) => axum::Json(serde_json::json!({
                                "status": "ok",
                                "path": rel_path,
                                "content": content
                            })),
                            Err(e) => axum::Json(serde_json::json!({
                                "status": "error",
                                "path": rel_path,
                                "message": format!("Fichier introuvable: {}", e)
                            }))
                        }
                    }
                }
            }),
        )
        .route(
            "/api/settings",
            axum::routing::get({
                let state = state_settings.clone();
                move || async move {
                    let exclusions = state.custom_exclusions.read().await.clone();
                    let catalogue = security_linter::SecurityLinter::get_supported_cve_rules();
                    let saved_active = state.active_cve_rules.read().await.clone();
                    let active_rules: Vec<String> = saved_active
                        .unwrap_or_else(|| catalogue.iter().map(|r| r.rule_id.clone()).collect());
                    axum::Json(SettingsPayload {
                        exclusions,
                        active_cve_rules: Some(active_rules),
                        cve_catalogue: Some(catalogue),
                        mcp_binary_path: resolve_mcp_binary_path(),
                    })
                }
            })
            .post({
                let state = state_settings.clone();
                move |axum::extract::Json(payload): axum::extract::Json<SettingsPayload>| async move {
                    {
                        let mut excl_guard = state.custom_exclusions.write().await;
                        *excl_guard = payload.exclusions.clone();
                    }
                    if let Some(ref rules) = payload.active_cve_rules {
                        let mut cve_guard = state.active_cve_rules.write().await;
                        *cve_guard = Some(rules.clone());
                    }
                    let p_root = state.project_root.read().await.clone();
                    let scanner = ProjectScanner::default();
                    let mut new_graph = scanner.scan_project_with_exclusions(&p_root, &payload.exclusions);
                    let active_opt = state.active_cve_rules.read().await.clone();
                    if let Some(ref allowed) = active_opt {
                        let allowed_set: std::collections::HashSet<&str> = allowed.iter().map(|s| s.as_str()).collect();
                        for node in &mut new_graph.nodes {
                            node.metadata.cve_anomalies.retain(|a| allowed_set.contains(a.rule_id.as_str()));
                            if node.metadata.cve_anomalies.is_empty() {
                                node.has_security_violation = false;
                            }
                        }
                    }
                    let mut g_guard = state.graph.write().await;
                    *g_guard = new_graph.clone();
                    state.graph_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let catalogue = security_linter::SecurityLinter::get_supported_cve_rules();
                    let active_resp = active_opt.unwrap_or_else(|| catalogue.iter().map(|r| r.rule_id.clone()).collect());
                    axum::Json(serde_json::json!({
                        "status": "ok",
                        "exclusions": payload.exclusions,
                        "active_cve_rules": active_resp,
                        "cve_catalogue": catalogue,
                        "total_nodes": new_graph.nodes.len(),
                        "total_edges": new_graph.edges.len()
                    }))
                }
            }),
        )
        .route("/api/health", axum::routing::get(|| async { "{\"status\":\"ok\",\"engine\":\"aether-v2\"}" }))
        .route(
            "/favicon.ico",
            axum::routing::get(|| async { axum::http::StatusCode::NO_CONTENT }),
        )
        // Pas de CORS permissif : le cockpit est servi par ce même serveur (same-origin) et le pont MCP
        // n'envoie pas d'Origin. Toute requête cross-origin / DNS-rebinding est refusée par le garde.
        .layer(axum::middleware::from_fn_with_state(
            local_guard::LocalGuard::new(args.port),
            local_guard::enforce_local_only,
        ));

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
