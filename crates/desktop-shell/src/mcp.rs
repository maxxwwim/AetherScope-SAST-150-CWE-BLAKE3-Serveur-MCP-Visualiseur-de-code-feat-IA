use crate::Args;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

pub async fn run_mcp_server(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let port = args.port;
    let path = args.path.clone();

    // 1. Détection intelligente : Si le serveur WebGL n'est pas déjà actif sur le port,
    // on le démarre automatiquement en tâche de fond pour garantir la disponibilité de l'API !
    let is_server_listening = {
        use std::net::{SocketAddr, TcpStream};
        use std::time::Duration;
        if let Ok(addr) = format!("127.0.0.1:{}", port).parse::<SocketAddr>() {
            TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok()
        } else {
            false
        }
    };

    if !is_server_listening {
        eprintln!("📡 [AetherScope MCP] Démarrage du serveur Axum & FileWatcher en arrière-plan sur http://127.0.0.1:{}...", port);
        let server_args = args.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::run_server_and_watcher(server_args).await {
                eprintln!("⚠️ [AetherScope MCP] Erreur serveur interne : {}", e);
            }
        });
        tokio::time::sleep(tokio::time::Duration::from_millis(350)).await;
    } else {
        eprintln!("📡 [AetherScope MCP] Connecté à l'instance active AetherScope sur http://127.0.0.1:{}", port);
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_res = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(json!({ "code": -32700, "message": format!("Parse error: {}", e) })),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err_res).unwrap());
                let _ = stdout.flush();
                continue;
            }
        };

        let resp = handle_mcp_request(req, port, &path).await;
        if let Some(res) = resp {
            let _ = writeln!(stdout, "{}", serde_json::to_string(&res).unwrap());
            let _ = stdout.flush();
        }
    }

    Ok(())
}

async fn handle_mcp_request(req: JsonRpcRequest, server_port: u16, default_path: &str) -> Option<JsonRpcResponse> {
    let id = req.id;

    match req.method.as_str() {
        "initialize" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "aetherscope-mcp",
                    "version": "2.0.0"
                }
            })),
            error: None,
        }),

        "notifications/initialized" => None,

        "ping" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        }),

        "tools/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "aetherscope_teleport_camera",
                        "description": "Oriente la caméra WebGL en direct dans AetherScope vers un symbole ou un fichier, et affiche où l'IA travaille en temps réel (fil d'Ariane violet, faisceau laser et réticule tactique).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": {
                                    "type": "string",
                                    "description": "Nom du symbole, fichier ou dossier cible (ex: 'RoomBooking', 'BookingEngine', 'contracts/packets.rs', 'server/api')"
                                },
                                "action": {
                                    "type": "string",
                                    "description": "Action en cours (ex: 'Inspection', 'Analyse', 'Création', 'Refactoring', 'Correction CVE', 'Audit', 'Tests')"
                                },
                                "details": {
                                    "type": "string",
                                    "description": "Courte description technique de ce que vous observez ou réalisez"
                                }
                            },
                            "required": ["target", "action", "details"]
                        }
                    },
                    {
                        "name": "aetherscope_get_graph_summary",
                        "description": "Récupère l'architecture complète du projet (nombre de nœuds, flux, liste des dossiers, santé globale et violations de sécurité CVE) depuis AetherScope.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "Chemin optionnel du projet à analyser (par défaut le projet actif)"
                                }
                            }
                        }
                    },
                    {
                        "name": "aetherscope_send_chat_message",
                        "description": "Envoie un message ou debrief technique de l'IA directement dans le chat intégré de l'interface AetherScope du développeur.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": {
                                    "type": "string",
                                    "description": "Contenu du message ou compte-rendu pour l'utilisateur"
                                },
                                "thought": {
                                    "type": "string",
                                    "description": "Raisonnement ou réflexion interne de l'IA (optionnel)"
                                },
                                "target_symbol": {
                                    "type": "string",
                                    "description": "Symbole ou fichier associé au message (optionnel)"
                                }
                            },
                            "required": ["text"]
                        }
                    },
                    {
                        "name": "aetherscope_get_chat_history",
                        "description": "Récupère l'historique des échanges récents du chat AetherScope.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "aetherscope_get_active_target",
                        "description": "Récupère le contexte ciblé et le handshake pour un symbole : appelants amont, dépendances aval, règles README EARS et imports interdits.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "symbol_id": {
                                    "type": "string",
                                    "description": "Nom ou identifiant du symbole ou fichier (ex: 'handle_booking', 'BookingForm')"
                                }
                            },
                            "required": ["symbol_id"]
                        }
                    },
                    {
                        "name": "aetherscope_scan_project",
                        "description": "Déclenche une réanalyse complète de l'architecture d'un projet pour rafraîchir le graphe 3D.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "Chemin du projet à réanalyser"
                                }
                            }
                        }
                    },
                    {
                        "name": "aetherscope_create_task",
                        "description": "Crée une nouvelle tâche (fonction, recherche, sécurité, refactorisation, etc.) avec son mini-README d'avancement pour structurer le travail dans le contexte du modèle et du développeur.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "title": {
                                    "type": "string",
                                    "description": "Titre explicite de la tâche à accomplir"
                                },
                                "kind": {
                                    "type": "string",
                                    "description": "Type de tâche : 'function', 'research', 'security_fix', 'arch_refactor', 'documentation', 'test', ou 'other'",
                                    "enum": ["function", "research", "security_fix", "arch_refactor", "documentation", "test", "other"]
                                },
                                "target_symbol": {
                                    "type": "string",
                                    "description": "Nom optionnel du symbole AST ciblé dans le graphe (ex: 'BookingService', 'verify_token')"
                                },
                                "target_file": {
                                    "type": "string",
                                    "description": "Chemin optionnel du fichier concerné (ex: 'src/auth.rs', 'contracts/ports.rs')"
                                },
                                "priority": {
                                    "type": "string",
                                    "description": "Priorité de la tâche ('low', 'medium', 'high', 'critical')",
                                    "enum": ["low", "medium", "high", "critical"]
                                },
                                "initial_readme": {
                                    "type": "string",
                                    "description": "Contenu Markdown optionnel pour initialiser le mini-README d'avancement (objectifs, checklist d'étapes)"
                                }
                            },
                            "required": ["title"]
                        }
                    },
                    {
                        "name": "aetherscope_update_task",
                        "description": "Met à jour l'état d'avancement d'une tâche, son statut ('pending', 'in_progress', 'blocked', 'done'), son pourcentage (0-100%) et ajoute des notes au mini-README.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": {
                                    "type": "string",
                                    "description": "Identifiant unique de la tâche (ex: 'task_123456_nom')"
                                },
                                "status": {
                                    "type": "string",
                                    "description": "Nouveau statut de la tâche ('pending', 'in_progress', 'blocked', 'done', 'cancelled')",
                                    "enum": ["pending", "in_progress", "blocked", "done", "cancelled"]
                                },
                                "progress": {
                                    "type": "integer",
                                    "description": "Pourcentage d'avancement estimé de la tâche entre 0 et 100"
                                },
                                "progress_note": {
                                    "type": "string",
                                    "description": "Courte note de progrès à consigner dans le mini-README de la tâche"
                                },
                                "full_readme": {
                                    "type": "string",
                                    "description": "Nouveau contenu complet du mini-README de la tâche (si réécriture / mise à jour complète de la checklist)"
                                }
                            },
                            "required": ["task_id"]
                        }
                    },
                    {
                        "name": "aetherscope_list_tasks",
                        "description": "Récupère la liste des tâches du projet AetherScope avec leur type, statut, avancement et résumé du mini-README.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "status_filter": {
                                    "type": "string",
                                    "description": "Filtre optionnel : 'active' (par défaut, exclut les terminées), 'all' (toutes les tâches), 'done' (uniquement terminées)",
                                    "enum": ["active", "all", "done"]
                                }
                            }
                        }
                    }
                ]
            })),
            error: None,
        }),


        "tools/call" => {
            let params = req.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            let result_content = match tool_name {
                "aetherscope_teleport_camera" => {
                    let target = arguments.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let action = arguments.get("action").and_then(|v| v.as_str()).unwrap_or("Inspection").to_string();
                    let details = arguments.get("details").and_then(|v| v.as_str()).unwrap_or("").to_string();

                    let res = send_http_request(server_port, "POST", "/api/ai/telemetry", Some(&json!({
                        "target": target,
                        "action": action,
                        "details": details
                    }))).await;

                    match res {
                        Ok(_) => json!([{
                            "type": "text",
                            "text": format!("🪐 Caméra AetherScope alignée sur : {} ({})\n📡 Faisceau quantique et fil d'Ariane stellaire illuminés sur le graphe 3D WebGL 2.", target, action)
                        }]),
                        Err(e) => json!([{
                            "type": "text",
                            "text": format!("⚠️ Télémétrie enregistrée : [{}] sur {}\n(Détails : {})", action, target, e)
                        }]),
                    }
                }

                "aetherscope_get_graph_summary" => {
                    let req_path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or(default_path);
                    let res = send_http_request(server_port, "GET", "/api/graph", None).await;

                    match res {
                        Ok(graph_val) => {
                            let summary = format_graph_summary(&graph_val);
                            json!([{ "type": "text", "text": summary }])
                        }
                        Err(_) => {
                            // Fallback direct AST scanner offline : scanne directement le projet sans dépendre du serveur web
                            let p_buf = PathBuf::from(req_path);
                            let scanner = ast_scanner::ProjectScanner::default();
                            let g = scanner.scan_project(&p_buf);
                            let summary = format_ast_summary(&g, &p_buf);
                            json!([{ "type": "text", "text": summary }])
                        }
                    }
                }

                "aetherscope_send_chat_message" => {
                    let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let thought = arguments.get("thought").and_then(|v| v.as_str()).unwrap_or("");
                    let target = arguments.get("target_symbol").and_then(|v| v.as_str()).map(|s| s.to_string());

                    let full_payload = if !thought.is_empty() {
                        format!("<thought>\n{}\n</thought>\n{}", thought, text)
                    } else {
                        text
                    };

                    let res = send_http_request(server_port, "POST", "/api/ai/chat", Some(&json!({
                        "text": full_payload,
                        "sender": "ai",
                        "target_symbol": target
                    }))).await;

                    match res {
                        Ok(_) => json!([{ "type": "text", "text": "💬 Message affiché dans le HUD AetherScope de l'utilisateur." }]),
                        Err(e) => json!([{ "type": "text", "text": format!("⚠️ Erreur envoi chat AetherScope : {}", e) }]),
                    }
                }

                "aetherscope_get_chat_history" => {
                    let res = send_http_request(server_port, "GET", "/api/ai/chat", None).await;
                    match res {
                        Ok(val) => json!([{ "type": "text", "text": serde_json::to_string_pretty(&val).unwrap_or_default() }]),
                        Err(e) => json!([{ "type": "text", "text": format!("⚠️ Erreur historique chat : {}", e) }]),
                    }
                }

                "aetherscope_get_active_target" => {
                    let symbol_id = arguments.get("symbol_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let res = send_http_request(server_port, "POST", "/api/ai/target", Some(&json!({
                        "symbol_id": symbol_id
                    }))).await;

                    match res {
                        Ok(val) => {
                            let text = format_active_target_response(&val);
                            json!([{ "type": "text", "text": text }])
                        }
                        Err(_) => {
                            let text = offline_active_target_fallback(default_path, &symbol_id);
                            json!([{ "type": "text", "text": text }])
                        }
                    }
                }

                "aetherscope_scan_project" => {
                    let scan_path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or(default_path);
                    let res = send_http_request(server_port, "POST", "/api/analyze", Some(&json!({
                        "path": scan_path
                    }))).await;

                    match res {
                        Ok(val) => json!([{ "type": "text", "text": format!("⚡ Projet réanalysé avec succès : {}", serde_json::to_string_pretty(&val).unwrap_or_default()) }]),
                        Err(e) => json!([{ "type": "text", "text": format!("⚠️ Erreur réanalyse : {}", e) }]),
                    }
                }

                "aetherscope_create_task" => {
                    let title = arguments.get("title").and_then(|v| v.as_str()).unwrap_or("Nouvelle tâche").to_string();
                    let kind_str = arguments.get("kind").and_then(|v| v.as_str()).unwrap_or("function").to_string();
                    let target_symbol = arguments.get("target_symbol").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let target_file = arguments.get("target_file").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let priority = arguments.get("priority").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let initial_readme = arguments.get("initial_readme").and_then(|v| v.as_str()).map(|s| s.to_string());

                    let payload = json!({
                        "title": title,
                        "kind": kind_str,
                        "target_symbol": target_symbol,
                        "target_file": target_file,
                        "author": "model",
                        "priority": priority,
                        "initial_readme": initial_readme
                    });

                    let res = send_http_request(server_port, "POST", "/api/tasks", Some(&payload)).await;
                    match res {
                        Ok(val) => {
                            let task_obj = val.get("task");
                            let task_id = task_obj.and_then(|t| t.get("id")).and_then(|i| i.as_str()).unwrap_or("");
                            json!([{
                                "type": "text",
                                "text": format!("📋 Tâche `{}` créée avec succès !\n📁 Mini-README disponible dans `.aether/tasks/{}/README.md`\n{}", task_id, task_id, serde_json::to_string_pretty(&val).unwrap_or_default())
                            }])
                        }
                        Err(_) => {
                            let p_buf = PathBuf::from(default_path);
                            let kind = ai_file_bridge::TaskKind::from_str_loose(&kind_str);
                            match ai_file_bridge::TaskManager::create_task(
                                &p_buf,
                                &title,
                                kind,
                                target_symbol.as_deref(),
                                target_file.as_deref(),
                                "model",
                                priority.as_deref(),
                                initial_readme.as_deref(),
                            ) {
                                Ok(t) => json!([{
                                    "type": "text",
                                    "text": format!("📋 Tâche `{}` créée (mode offline) !\n📁 Fichier : `.aether/tasks/{}/README.md`", t.id, t.id)
                                }]),
                                Err(e) => json!([{ "type": "text", "text": format!("⚠️ Erreur création tâche : {}", e) }]),
                            }
                        }
                    }
                }

                "aetherscope_update_task" => {
                    let task_id = arguments.get("task_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let status_str = arguments.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let progress = arguments.get("progress").and_then(|v| v.as_u64()).map(|p| p as u8);
                    let progress_note = arguments.get("progress_note").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let full_readme = arguments.get("full_readme").and_then(|v| v.as_str()).map(|s| s.to_string());

                    let payload = json!({
                        "task_id": task_id,
                        "status": status_str,
                        "progress": progress,
                        "progress_note": progress_note,
                        "full_readme": full_readme
                    });

                    let res = send_http_request(server_port, "POST", "/api/tasks/update", Some(&payload)).await;
                    match res {
                        Ok(val) => json!([{
                            "type": "text",
                            "text": format!("⚡ Tâche `{}` mise à jour avec succès :\n{}", task_id, serde_json::to_string_pretty(&val).unwrap_or_default())
                        }]),
                        Err(_) => {
                            let p_buf = PathBuf::from(default_path);
                            let status = status_str.as_deref().map(ai_file_bridge::TaskStatus::from_str_loose);
                            match ai_file_bridge::TaskManager::update_task(
                                &p_buf,
                                &task_id,
                                status,
                                progress,
                                progress_note.as_deref(),
                                full_readme.as_deref(),
                            ) {
                                Ok(Some(t)) => json!([{
                                    "type": "text",
                                    "text": format!("⚡ Tâche `{}` mise à jour (mode offline). Statut: {:?}, Progrès: {}%", t.id, t.status, t.progress)
                                }]),
                                Ok(None) => json!([{ "type": "text", "text": format!("⚠️ Tâche `{}` introuvable.", task_id) }]),
                                Err(e) => json!([{ "type": "text", "text": format!("⚠️ Erreur mise à jour tâche : {}", e) }]),
                            }
                        }
                    }
                }

                "aetherscope_list_tasks" => {
                    let filter = arguments.get("status_filter").and_then(|v| v.as_str()).unwrap_or("active");
                    let res = send_http_request(server_port, "GET", "/api/tasks", None).await;

                    let tasks = match res {
                        Ok(val) => {
                            if let Some(arr) = val.get("tasks").and_then(|t| t.as_array()) {
                                arr.clone()
                            } else {
                                vec![]
                            }
                        }
                        Err(_) => {
                            let p_buf = PathBuf::from(default_path);
                            let raw_tasks = ai_file_bridge::TaskManager::list_tasks(&p_buf);
                            serde_json::to_value(&raw_tasks).ok().and_then(|v| v.as_array().cloned()).unwrap_or_default()
                        }
                    };

                    let filtered: Vec<&Value> = tasks.iter().filter(|t| {
                        let st = t.get("status").and_then(|s| s.as_str()).unwrap_or("");
                        match filter {
                            "all" => true,
                            "done" => st == "done",
                            _ => st != "done" && st != "cancelled",
                        }
                    }).collect();

                    if filtered.is_empty() {
                        json!([{ "type": "text", "text": "📋 Aucune tâche trouvée pour ce filtre." }])
                    } else {
                        let mut output = format!("📋 Tâches AetherScope (Total : {}):\n\n", filtered.len());
                        for t in filtered {
                            let id = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            let title = t.get("title").and_then(|v| v.as_str()).unwrap_or("");
                            let kind = t.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                            let status = t.get("status").and_then(|v| v.as_str()).unwrap_or("");
                            let progress = t.get("progress").and_then(|v| v.as_u64()).unwrap_or(0);
                            let target = t.get("target_symbol").and_then(|v| v.as_str()).unwrap_or("-");
                            let summary = t.get("readme_summary").and_then(|v| v.as_str()).unwrap_or("");

                            output.push_str(&format!("• `{}` [{}] **{}** ({}, {}%)\n  - Cible : `{}`\n  - Avancement : {}\n\n", id, kind, title, status, progress, target, summary));
                        }
                        json!([{ "type": "text", "text": output }])
                    }
                }

                _ => json!([{ "type": "text", "text": format!("Outil inconnu : {}", tool_name) }]),

            };

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({ "content": result_content })),
                error: None,
            })
        }

        _ => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(json!({ "code": -32601, "message": "Method not found" })),
        }),
    }
}

fn format_graph_summary(val: &Value) -> String {
    let nodes = val.get("nodes").and_then(|v| v.as_array());
    let edges = val.get("edges").and_then(|v| v.as_array());
    let total_nodes = nodes.map(|n| n.len()).unwrap_or(0);
    let total_edges = edges.map(|e| e.len()).unwrap_or(0);

    let mut folders = std::collections::BTreeMap::new();
    let mut violations = Vec::new();

    if let Some(nodes_arr) = nodes {
        for n in nodes_arr {
            let f = n.get("folder").and_then(|v| v.as_str()).unwrap_or("root");
            *folders.entry(f.to_string()).or_insert(0usize) += 1;
            if n.get("has_security_violation").and_then(|v| v.as_bool()).unwrap_or(false) {
                let name = n.get("name").and_then(|v| v.as_str()).unwrap_or("inconnu");
                let file = n.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
                violations.push(format!("- 🚨 {} ({})", name, file));
            }
        }
    }

    let mut folders_str = String::new();
    for (f, cnt) in folders.iter().take(15) {
        folders_str.push_str(&format!("  - 📁 `{}` : {} symbole(s)\n", f, cnt));
    }

    let violations_str = if violations.is_empty() {
        "✅ Aucune violation de sécurité ou CVE détectée.".to_string()
    } else {
        format!("⚠️ {} Violation(s) de sécurité / CVE détectée(s) :\n{}", violations.len(), violations.join("\n"))
    };

    format!(
        "🪐 Résumé Architectural AetherScope V2 :\n\
         • Total Nœuds : {}\n\
         • Total Relations (Arêtes) : {}\n\
         • Modules / Dossiers ({} au total) :\n{}\
         • Statut Sécurité & Règles :\n{}\n\
         • Visualiseur WebGL 2 disponible sur http://127.0.0.1:3030",
        total_nodes, total_edges, folders.len(), folders_str, violations_str
    )
}

fn format_ast_summary(g: &core_graph::graph::AetherGraph, project_root: &Path) -> String {
    let total_nodes = g.nodes.len();
    let total_edges = g.edges.len();

    let mut folders = std::collections::BTreeMap::new();
    for n in &g.nodes {
        let f = if n.folder.is_empty() { "root" } else { &n.folder };
        *folders.entry(f.to_string()).or_insert(0usize) += 1;
    }

    let mut folders_str = String::new();
    for (f, cnt) in folders.iter().take(15) {
        folders_str.push_str(&format!("  - 📁 `{}` : {} symbole(s)\n", f, cnt));
    }

    format!(
        "🪐 Résumé Architectural AetherScope V2 (Mode Autonome AST) :\n\
         • Racine : {}\n\
         • Total Nœuds : {}\n\
         • Total Relations : {}\n\
         • Modules / Dossiers :\n{}\
         • Note : Lancez `aether-engine` pour activer la vue WebGL 2 interactive en direct.",
        project_root.display(), total_nodes, total_edges, folders_str
    )
}

fn format_active_target_response(val: &Value) -> String {
    let status = val.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
    if status != "ok" {
        return "⚠️ Symbole ou module non trouvé dans le graphe actif AetherScope.".to_string();
    }

    if let Some(md) = val.get("formatted_doc_and_liaisons").and_then(|v| v.as_str()) {
        let handshake = val.get("handshake_xml").and_then(|v| v.as_str()).unwrap_or("");
        if !handshake.is_empty() {
            format!("{}\n\n### 🤝 Handshake Copilote IA :\n```xml\n{}\n```", md, handshake)
        } else {
            md.to_string()
        }
    } else {
        serde_json::to_string_pretty(val).unwrap_or_default()
    }
}

fn offline_active_target_fallback(project_path: &str, symbol_id: &str) -> String {
    let p_buf = PathBuf::from(project_path);
    let scanner = ast_scanner::ProjectScanner::default();
    let g = scanner.scan_project(&p_buf);

    let sym_opt = g.nodes.iter().find(|n| {
        n.id == symbol_id
            || n.name == symbol_id
            || n.file_path == symbol_id
            || n.id.ends_with(&format!("::{}", symbol_id))
    });

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

        let in_cat = core_graph::edge::CategorizedLiaisons::from_edges(&incoming_edges, &sym.id, true);
        let out_cat = core_graph::edge::CategorizedLiaisons::from_edges(&outgoing_edges, &sym.id, false);

        let doc_display = if !sym.docstring.is_empty() {
            &sym.docstring
        } else if let Some(raw) = &sym.metadata.raw_content {
            raw.as_str()
        } else {
            "*(Aucun commentaire AST direct détecté)*"
        };

        let callers_md = in_cat.to_compact_markdown("amont", 4);
        let deps_md = out_cat.to_compact_markdown("aval", 4);

        format!(
            "# 🎯 Cible : `{}` (Mode Autonome AST)\n- **Fichier :** `{}:{}`\n- **Architecture :** Galaxie `{}` | Couche `{}` | Stéréotype `{}`\n\n## 📖 DOCUMENTATION & SPÉCIFICATIONS AST :\n{}\n\n## 📥 FLUX AMONT / APPELANTS (Total: {}):\n{}\n## 📤 FLUX AVAL / DÉPENDANCES & LIAISONS (Total: {}):\n{}",
            sym.name,
            sym.file_path,
            sym.line_number,
            sym.galaxy,
            sym.layer,
            sym.stereotype,
            doc_display,
            in_cat.total(),
            callers_md,
            out_cat.total(),
            deps_md
        )
    } else {
        format!("⚠️ Symbole '{}' introuvable dans le projet '{}'.", symbol_id, project_path)
    }
}

async fn send_http_request(
    server_port: u16,
    method: &str,
    path: &str,
    body: Option<&Value>,
) -> Result<Value, String> {
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    let addr: SocketAddr = format!("127.0.0.1:{}", server_port)
        .parse()
        .map_err(|e| format!("Adresse invalide: {}", e))?;

    let method_str = method.to_string();
    let path_str = path.to_string();
    let body_str = body.map(|b| serde_json::to_string(b).unwrap_or_default()).unwrap_or_default();

    tokio::task::spawn_blocking(move || {
        let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(800))
            .map_err(|e| format!("AetherScope non connecté sur http://127.0.0.1:{} ({})", addr.port(), e))?;

        stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
        stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;

        let req = format!(
            "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            method_str,
            path_str,
            addr.port(),
            body_str.len(),
            body_str
        );

        stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).map_err(|e| e.to_string())?;

        let response_str = String::from_utf8_lossy(&response_bytes);
        if let Some((_, json_part)) = response_str.split_once("\r\n\r\n") {
            let val: Value = serde_json::from_str(json_part.trim()).unwrap_or_else(|_| json!({ "raw": json_part }));
            Ok(val)
        } else {
            Ok(json!({ "raw": response_str }))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
