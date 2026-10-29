use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

const EXCLUDED_WATCH_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "dist",
    "build",
    "out",
    "w64devkit",
    "mingw64",
    "mingw32",
    "res",
    "resources",
    "assets",
    "imgs",
    "images",
    "icons",
    "fonts",
    "country_flags",
    "libs",
    "lang",
    "locales",
    "i18n",
    "vendor",
    "third_party",
    "client_apk",
    "client_c",
    "cpp_builder",
    "python_utils_testing",
    "cosmoscope_output",
    "coverage",
    "venv",
    "__pycache__",
];

fn is_excluded_dir_name(name: &str) -> bool {
    name.starts_with('.')
        || name.contains("WebView2")
        || EXCLUDED_WATCH_DIRS
            .iter()
            .any(|d| name.eq_ignore_ascii_case(d))
}

fn watch_filtered_tree(
    watcher: &mut RecommendedWatcher,
    dir: &Path,
    depth: usize,
) -> Result<(), notify::Error> {
    if !dir.is_dir() {
        return Ok(());
    }
    if depth >= 2 {
        let _ = watcher.watch(dir, RecursiveMode::Recursive);
        return Ok(());
    }
    let _ = watcher.watch(dir, RecursiveMode::NonRecursive);
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !is_excluded_dir_name(&name) {
                    let _ = watch_filtered_tree(watcher, &path, depth + 1);
                }
            }
        }
    }
    Ok(())
}

pub struct FileWatcherBridge {
    watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    pub project_root: PathBuf,
}

impl FileWatcherBridge {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, notify::Error> {
        let root_buf = root.as_ref().to_path_buf();
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_millis(200)),
        )?;

        watch_filtered_tree(&mut watcher, root.as_ref(), 0)?;

        Ok(Self {
            watcher,
            rx,
            project_root: root_buf,
        })
    }

    /// Met à jour dynamiquement le dossier racine surveillé lorsque l'utilisateur change de projet
    pub fn switch_root<P: AsRef<Path>>(&mut self, new_root: P) -> Result<(), notify::Error> {
        let new_buf = new_root.as_ref().to_path_buf();
        if self.project_root != new_buf && new_buf.is_dir() {
            let _ = self.watcher.unwatch(&self.project_root);
            watch_filtered_tree(&mut self.watcher, &new_buf, 0)?;
            self.project_root = new_buf;
        }
        Ok(())
    }

    /// Récupère les chemins de fichiers modifiés lors de cette boucle
    pub fn poll_changed_files(&self) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        while let Ok(res) = self.rx.try_recv() {
            if let Ok(event) = res {
                if event.kind.is_modify() || event.kind.is_create() {
                    for path in event.paths {
                        if !path.is_file() {
                            continue;
                        }
                        let p_str = path.to_string_lossy();
                        if p_str.ends_with(".log")
                            || p_str.ends_with("aetherscope_viewer.html")
                            || p_str.ends_with("AETHER_AI_RULES.md")
                        {
                            continue;
                        }
                        let has_excluded_comp = path.components().any(|c| {
                            let s = c.as_os_str().to_string_lossy();
                            is_excluded_dir_name(&s)
                        });
                        if has_excluded_comp {
                            continue;
                        }
                        let ext = path
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        if [
                            "rs", "ts", "tsx", "js", "jsx", "py", "md", "cpp", "hpp", "cc", "hh",
                            "c", "h",
                        ]
                        .contains(&ext.as_str())
                        {
                            if !changed.contains(&path) {
                                changed.push(path);
                            }
                        }
                    }
                }
            }
        }
        changed
    }
}
