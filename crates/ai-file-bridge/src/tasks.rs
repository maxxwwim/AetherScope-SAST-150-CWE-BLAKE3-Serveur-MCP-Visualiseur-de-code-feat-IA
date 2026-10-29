use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    Function,
    Research,
    SecurityFix,
    ArchRefactor,
    Documentation,
    Test,
    Other,
}

impl Default for TaskKind {
    fn default() -> Self {
        TaskKind::Function
    }
}

impl TaskKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskKind::Function => "function",
            TaskKind::Research => "research",
            TaskKind::SecurityFix => "security_fix",
            TaskKind::ArchRefactor => "arch_refactor",
            TaskKind::Documentation => "documentation",
            TaskKind::Test => "test",
            TaskKind::Other => "other",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "function" | "func" | "code" | "feat" | "feature" => TaskKind::Function,
            "research" | "recherche" | "investigation" | "audit_explore" => TaskKind::Research,
            "security" | "security_fix" | "cve" | "sast" => TaskKind::SecurityFix,
            "refactor" | "arch" | "arch_refactor" | "architecture" => TaskKind::ArchRefactor,
            "doc" | "documentation" | "spec" => TaskKind::Documentation,
            "test" | "tests" | "unit_test" => TaskKind::Test,
            _ => TaskKind::Other,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TaskKind::Function => "⚙️",
            TaskKind::Research => "🔍",
            TaskKind::SecurityFix => "🛡️",
            TaskKind::ArchRefactor => "🏛️",
            TaskKind::Documentation => "📖",
            TaskKind::Test => "🧪",
            TaskKind::Other => "📌",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Done => "done",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "in_progress" | "inprogress" | "doing" | "active" => TaskStatus::InProgress,
            "blocked" | "bloque" | "bloqué" | "waiting" => TaskStatus::Blocked,
            "done" | "completed" | "fini" | "resolu" => TaskStatus::Done,
            "cancelled" | "abandoned" | "annule" | "annulé" => TaskStatus::Cancelled,
            _ => TaskStatus::Pending,
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "⏳ EN ATTENTE",
            TaskStatus::InProgress => "⚡ EN COURS",
            TaskStatus::Blocked => "⛔ BLOQUÉ",
            TaskStatus::Done => "✅ TERMINÉ",
            TaskStatus::Cancelled => "⚪ ANNULÉ",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub kind: TaskKind,
    #[serde(default)]
    pub status: TaskStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_file: Option<String>,
    #[serde(default = "default_author")]
    pub author: String, // "user" | "model"
    #[serde(default = "default_priority")]
    pub priority: String, // "low" | "medium" | "high" | "critical"
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub progress: u8, // 0..=100
    #[serde(default)]
    pub readme_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_readme: Option<String>,
}

fn default_author() -> String {
    "user".to_string()
}

fn default_priority() -> String {
    "medium".to_string()
}

pub struct TaskManager;

impl TaskManager {
    /// Retourne le chemin du dossier `.aether/tasks`
    pub fn tasks_dir(project_root: &Path) -> PathBuf {
        project_root.join(".aether").join("tasks")
    }

    /// Liste toutes les tâches existantes
    pub fn list_tasks(project_root: &Path) -> Vec<TaskItem> {
        let dir = Self::tasks_dir(project_root);
        if !dir.exists() {
            return Vec::new();
        }

        let mut tasks = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let task_folder = entry.path();
                        let meta_file = task_folder.join("task.json");
                        let readme_file = task_folder.join("README.md");

                        if meta_file.exists() {
                            if let Ok(json_str) = fs::read_to_string(&meta_file) {
                                if let Ok(mut task) = serde_json::from_str::<TaskItem>(&json_str) {
                                    if readme_file.exists() {
                                        if let Ok(readme_content) = fs::read_to_string(&readme_file) {
                                            task.full_readme = Some(readme_content.clone());
                                            task.readme_summary = extract_summary(&readme_content);
                                        }
                                    }
                                    tasks.push(task);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Tri par date de mise à jour descendante
        tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        tasks
    }

    /// Récupère une tâche spécifique par son ID
    pub fn get_task(project_root: &Path, task_id: &str) -> Option<TaskItem> {
        let folder = Self::tasks_dir(project_root).join(task_id);
        let meta_file = folder.join("task.json");
        let readme_file = folder.join("README.md");

        if !meta_file.exists() {
            return None;
        }

        let json_str = fs::read_to_string(&meta_file).ok()?;
        let mut task = serde_json::from_str::<TaskItem>(&json_str).ok()?;

        if readme_file.exists() {
            if let Ok(readme_content) = fs::read_to_string(&readme_file) {
                task.full_readme = Some(readme_content.clone());
                task.readme_summary = extract_summary(&readme_content);
            }
        }

        Some(task)
    }

    /// Crée une nouvelle tâche et initialise son mini-README
    pub fn create_task(
        project_root: &Path,
        title: &str,
        kind: TaskKind,
        target_symbol: Option<&str>,
        target_file: Option<&str>,
        author: &str,
        priority: Option<&str>,
        initial_readme: Option<&str>,
    ) -> std::io::Result<TaskItem> {
        let tasks_dir = Self::tasks_dir(project_root);
        fs::create_dir_all(&tasks_dir)?;

        let now_dt = Local::now();
        let now_str = now_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let slug = slugify(title);
        let task_id = format!("task_{}_{}", timestamp_ms % 1_000_000, slug);
        let task_folder = tasks_dir.join(&task_id);
        fs::create_dir_all(&task_folder)?;

        let target_sym_owned = target_symbol.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let target_file_owned = target_file.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let prio = priority.unwrap_or("medium").to_string();

        let readme_content = match initial_readme {
            Some(r) if !r.trim().is_empty() => r.trim().to_string(),
            _ => generate_default_readme(title, &kind, target_sym_owned.as_deref(), target_file_owned.as_deref(), &now_str),
        };

        let readme_summary = extract_summary(&readme_content);

        let task = TaskItem {
            id: task_id.clone(),
            title: title.trim().to_string(),
            kind,
            status: TaskStatus::Pending,
            target_symbol: target_sym_owned,
            target_file: target_file_owned,
            author: author.trim().to_string(),
            priority: prio,
            created_at: now_str.clone(),
            updated_at: now_str,
            progress: 0,
            readme_summary,
            full_readme: Some(readme_content.clone()),
        };

        // Sauvegarde task.json
        let meta_file = task_folder.join("task.json");
        fs::write(&meta_file, serde_json::to_string_pretty(&task)?)?;

        // Sauvegarde README.md
        let readme_file = task_folder.join("README.md");
        fs::write(&readme_file, &readme_content)?;

        // Régénération de TASKS_SUMMARY.md
        let _ = Self::sync_tasks_summary_markdown(project_root);

        Ok(task)
    }

    /// Met à jour l'état d'une tâche, son avancement ou complète son README
    pub fn update_task(
        project_root: &Path,
        task_id: &str,
        status: Option<TaskStatus>,
        progress: Option<u8>,
        append_note: Option<&str>,
        full_readme: Option<&str>,
    ) -> std::io::Result<Option<TaskItem>> {
        let task_folder = Self::tasks_dir(project_root).join(task_id);
        let meta_file = task_folder.join("task.json");
        let readme_file = task_folder.join("README.md");

        if !meta_file.exists() {
            return Ok(None);
        }

        let json_str = fs::read_to_string(&meta_file)?;
        let mut task: TaskItem = serde_json::from_str(&json_str)?;

        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        task.updated_at = now_str.clone();

        if let Some(new_status) = status {
            task.status = new_status;
            if task.status == TaskStatus::Done && task.progress < 100 {
                task.progress = 100;
            }
        }

        if let Some(p) = progress {
            task.progress = p.min(100);
            if task.progress == 100 && task.status == TaskStatus::Pending {
                task.status = TaskStatus::Done;
            } else if task.progress > 0 && task.status == TaskStatus::Pending {
                task.status = TaskStatus::InProgress;
            }
        }

        let mut current_readme = if readme_file.exists() {
            fs::read_to_string(&readme_file).unwrap_or_default()
        } else {
            String::new()
        };

        if let Some(new_readme) = full_readme {
            current_readme = new_readme.to_string();
        }

        if let Some(note) = append_note {
            if !note.trim().is_empty() {
                current_readme.push_str(&format!("\n\n### 📝 Note d'avancement ({})\n{}\n", now_str, note.trim()));
            }
        }

        task.readme_summary = extract_summary(&current_readme);
        task.full_readme = Some(current_readme.clone());

        // Réécriture
        fs::write(&meta_file, serde_json::to_string_pretty(&task)?)?;
        fs::write(&readme_file, &current_readme)?;

        // Régénération de TASKS_SUMMARY.md
        let _ = Self::sync_tasks_summary_markdown(project_root);

        Ok(Some(task))
    }

    /// Supprime une tâche
    pub fn delete_task(project_root: &Path, task_id: &str) -> std::io::Result<bool> {
        let task_folder = Self::tasks_dir(project_root).join(task_id);
        if task_folder.exists() {
            fs::remove_dir_all(&task_folder)?;
            let _ = Self::sync_tasks_summary_markdown(project_root);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Régénère le fichier synthèse `.aether/TASKS_SUMMARY.md` pour l'IA et l'utilisateur
    pub fn sync_tasks_summary_markdown(project_root: &Path) -> std::io::Result<String> {
        let aether_dir = project_root.join(".aether");
        fs::create_dir_all(&aether_dir)?;

        let summary_file = aether_dir.join("TASKS_SUMMARY.md");
        let tasks = Self::list_tasks(project_root);
        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut md = String::new();
        md.push_str("# 📋 AETHERSCOPE — JOURNAL DE BORD DES TÂCHES DU PROJET\n\n");
        md.push_str(&format!("> Synchronisé le `{}` | Total Tâches : `{}`\n\n", now_str, tasks.len()));

        if tasks.is_empty() {
            md.push_str("*(Aucune tâche active dans le cockpit AetherScope. L'IA ou l'utilisateur peut en ajouter via `aetherscope_create_task`)*\n");
        } else {
            let active: Vec<&TaskItem> = tasks.iter().filter(|t| t.status != TaskStatus::Done && t.status != TaskStatus::Cancelled).collect();
            let completed: Vec<&TaskItem> = tasks.iter().filter(|t| t.status == TaskStatus::Done).collect();

            if !active.is_empty() {
                md.push_str(&format!("## ⚡ Tâches en cours & à faire ({})\n\n", active.len()));
                md.push_str("| ID | Type | Statut | Progrès | Titre | Cible | Auteur |\n");
                md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
                for t in active {
                    let target = t.target_symbol.as_deref().or(t.target_file.as_deref()).unwrap_or("-");
                    md.push_str(&format!(
                        "| `{}` | {} {} | {} | {}% | **{}** | `{}` | {} |\n",
                        t.id, t.kind.icon(), t.kind.as_str(), t.status.badge(), t.progress, t.title, target, t.author
                    ));
                }
                md.push_str("\n");
            }

            if !completed.is_empty() {
                md.push_str(&format!("## ✅ Tâches terminées ({})\n\n", completed.len()));
                for t in completed {
                    let target = t.target_symbol.as_deref().or(t.target_file.as_deref()).unwrap_or("-");
                    md.push_str(&format!("- [x] **{}** (`{}`) — {} (*{}*)\n", t.title, t.id, target, t.updated_at));
                }
                md.push_str("\n");
            }

            md.push_str("## 📖 Détails des tâches actives :\n\n");
            for t in tasks.iter().filter(|t| t.status != TaskStatus::Done && t.status != TaskStatus::Cancelled).take(5) {
                md.push_str(&format!("### {} `{}` : {}\n", t.kind.icon(), t.id, t.title));
                md.push_str(&format!("- **Statut :** {} ({}%)\n", t.status.badge(), t.progress));
                if let Some(target) = &t.target_symbol {
                    md.push_str(&format!("- **Symbole cible :** `{}`\n", target));
                }
                if let Some(file) = &t.target_file {
                    md.push_str(&format!("- **Fichier cible :** `{}`\n", file));
                }
                if let Some(readme) = &t.full_readme {
                    md.push_str("\n```markdown\n");
                    md.push_str(readme.trim());
                    md.push_str("\n```\n\n");
                }
            }
        }

        fs::write(&summary_file, &md)?;
        Ok(md)
    }
}

fn extract_summary(readme: &str) -> String {
    let mut lines = Vec::new();
    for line in readme.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        lines.push(trimmed);
        if lines.len() >= 3 {
            break;
        }
    }
    if lines.is_empty() {
        "Aucun détail d'avancement pour le moment.".to_string()
    } else {
        lines.join(" ")
    }
}

fn slugify(text: &str) -> String {
    let clean: String = text
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect();
    let parts: Vec<&str> = clean.split('_').filter(|s| !s.is_empty()).take(4).collect();
    if parts.is_empty() {
        "task".to_string()
    } else {
        parts.join("_")
    }
}

fn generate_default_readme(
    title: &str,
    kind: &TaskKind,
    target_symbol: Option<&str>,
    target_file: Option<&str>,
    created_at: &str,
) -> String {
    let kind_icon = kind.icon();
    let kind_title = match kind {
        TaskKind::Function => "Implémentation / Refactorisation de Fonction",
        TaskKind::Research => "Recherche & Exploration d'Architecture",
        TaskKind::SecurityFix => "Correction de Sécurité & Garde-fous",
        TaskKind::ArchRefactor => "Refactorisation Architecturale & Couplage",
        TaskKind::Documentation => "Documentation & Spécifications EARS",
        TaskKind::Test => "Tests Unitaires & Intégration",
        TaskKind::Other => "Tâche de Développement",
    };

    let target_str = match (target_symbol, target_file) {
        (Some(s), Some(f)) => format!("`{}` ({})", s, f),
        (Some(s), None) => format!("`{}`", s),
        (None, Some(f)) => format!("`{}`", f),
        (None, None) => "Projet Global".to_string(),
    };

    format!(
        r#"# {} {} : {}

- **Cible :** {}
- **Créée le :** `{}`
- **Statut Initial :** `⏳ EN ATTENTE (0%)`

---

## 🎯 Objectif
{}

## 📋 Plan d'Action & Étapes
- [ ] 1. Analyse initiale des dépendances et contrats
- [ ] 2. Conception de la solution chirurgicale (< 35 lignes par fonction)
- [ ] 3. Validation de l'audit de sécurité (`LIVE_AUDIT.md`)

## 📝 Journal d'Avancement
*(Initialisé. L'IA ou l'utilisateur mettra à jour cette section au fur et à mesure)*
"#,
        kind_icon, kind_title, title, target_str, created_at, title
    )
}
