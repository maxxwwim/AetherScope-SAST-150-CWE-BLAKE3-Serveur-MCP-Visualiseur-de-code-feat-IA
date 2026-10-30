use core_graph::bubble::{AstAttribute, AstMethod, CveAnomalyRecord, SymbolBubble, SymbolKind};
use core_graph::edge::{AffiliationEdge, EdgeType, FilterAffiliation};
use core_graph::graph::AetherGraph;
use core_graph::layout::compute_all_layouts;
use core_graph::manifest::ProjectManifest;
use rayon::prelude::*;
use regex::Regex;
use security_linter::contract::ReadmeContract;
use security_linter::linter::SecurityLinter;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use walkdir::WalkDir;

macro_rules! static_regex {
    ($name:ident, $pattern:expr) => {
        fn $name() -> &'static Regex {
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(|| Regex::new($pattern).expect("Invalid static regex"))
        }
    };
}

static_regex!(
    re_complexity,
    r#"(?:\b(?:if|else|elif|match|for|while|loop|catch|case)\b|&&|\|\|)"#
);
static_regex!(
    re_js_import,
    r#"import\s+(?:(\{[^}]+\})|([A-Za-z0-9_]+)|\*\s+as\s+([A-Za-z0-9_]+))?\s*(?:from\s*)?['"]([^'"]+)['"]"#
);
static_regex!(re_js_require, r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#);
static_regex!(
    re_js_class,
    r#"(?:export\s+)?(?:class|interface)\s+([A-Za-z0-9_]+)(?:\s+extends\s+([A-Za-z0-9_]+))?"#
);
static_regex!(
    re_js_func,
    r#"(?:export\s+(?:default\s+)?)?(?:async\s+)?function\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)"#
);
static_regex!(
    re_js_arrow,
    r#"(?:export\s+)?const\s+([A-Za-z0-9_]+)\s*=\s*(?:async\s*)?\(([^)]*)\)\s*=>"#
);
static_regex!(re_call_ident, r#"\b([A-Za-z0-9_]+)\s*\("#);
static_regex!(re_jsx_tag, r#"<([A-Z][A-Za-z0-9_]+)"#);
static_regex!(
    re_js_network,
    r#"(?:fetch|axios\.(?:get|post|put|delete|patch)|api(?:Client)?\.(?:get|post|put|delete))\s*\(\s*['"`]([^'"`]+)['"`]"#
);
static_regex!(re_js_submit, r#"onSubmit\s*=\s*\{\s*([A-Za-z0-9_]+)"#);
static_regex!(re_js_hook, r#"(use[A-Z][A-Za-z0-9_]*)"#);
static_regex!(re_js_ws_emit, r#"socket\.emit\(['"`]([^'"`]+)['"`]"#);

static_regex!(
    re_py_import,
    r#"(?:^|\n)\s*(?:from\s+([a-zA-Z0-9_\.]+)\s+import\s+([a-zA-Z0-9_,\s\*]+)|import\s+([a-zA-Z0-9_,\s]+))"#
);
static_regex!(
    re_py_class,
    r#"(?:^|\n)class\s+([A-Za-z0-9_]+)(?:\(([^)]*)\))?:"#
);
static_regex!(
    re_py_field,
    r#"(?:^|\n)\s+([a-zA-Z0-9_]+)\s*:\s*([A-Za-z0-9_\[\], ]+)"#
);
static_regex!(
    re_py_func,
    r#"(?:^|\n)(?:async\s+)?def\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)"#
);
static_regex!(re_py_route_str, r#"['"](/api/[^'"]+)['"]"#);

static_regex!(
    re_rs_mod,
    r#"(?:^|\n)\s*(?:pub\s+)?mod\s+([a-zA-Z0-9_]+)\s*;"#
);
static_regex!(re_rs_use, r#"(?:^|\n)\s*(?:pub\s+)?use\s+([^;]+);"#);
static_regex!(
    re_rs_struct,
    r#"(?:^|\n)\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait)\s+([A-Za-z0-9_]+)"#
);
static_regex!(
    re_rs_field,
    r#"(?:^|\n)\s*(?:pub(?:\([^)]*\))?\s+)?([a-z0-9_]+)\s*:\s*([^,\n}]+)"#
);
static_regex!(
    re_rs_impl_trait,
    r#"(?:^|\n)\s*impl(?:<[^>]*>)?\s+([A-Za-z0-9_:]+)\s+for\s+([A-Za-z0-9_]+)"#
);
static_regex!(
    re_rs_fn,
    r#"(?:^|\n)\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*(?:<[^>]*>)?\s*\(([^)]*)\)(?:\s*->\s*([^{\n]+))?"#
);
static_regex!(re_rs_qual_call, r#"\b([A-Za-z0-9_]+)::([a-zA-Z0-9_]+)\s*\("#);
static_regex!(
    re_rs_route,
    r#"fetch\(['"`]([^'"`]+)['"`]|route\(\s*['"`]([^'"`]+)['"`]"#
);
static_regex!(
    re_rs_ws,
    r#"(ClientPacket::[A-Za-z0-9_]+|ServerPacket::[A-Za-z0-9_]+)"#
);

static_regex!(re_cpp_include, r#"#include\s*([<"])([^>"]+)[>"]"#);
static_regex!(
    re_cpp_class,
    r#"(?:template\s*<[^>]*>\s*)?(?:class|struct)\s+([A-Za-z0-9_]+)(?:\s*:\s*([^{;]+))?\s*\{"#
);
static_regex!(
    re_cpp_member_method,
    r#"(?:^|\n)\s*(?:(?:virtual|static|inline|explicit)\s+)*(?:[A-Za-z0-9_:<>&*~]+\s+)+([A-Za-z0-9_]+)\s*\(([^)]*)\)"#
);
static_regex!(
    re_cpp_member_field,
    r#"(?:^|\n)\s*(?:[A-Za-z0-9_:<>&*]+\s+)+([A-Za-z0-9_]+)\s*;"#
);
static_regex!(
    re_cpp_func,
    r#"(?:^|\n)\s*(?:(?:virtual|static|inline|explicit|constexpr|friend)\s+)*(?:[A-Za-z0-9_:<>&*~]+\s+)+([A-Za-z0-9_]+(?:::[A-Za-z0-9_]+)?)\s*\(([^)]*)\)\s*(?:const|override|noexcept)?\s*\{"#
);
static_regex!(
    re_cpp_call,
    r#"\b([A-Za-z0-9_]+(?:::[A-Za-z0-9_]+)?)\s*\("#
);

static_regex!(
    re_sol_import,
    r#"import\s+(?:(?:\{[^}]+\}|\*\s+as\s+[A-Za-z0-9_]+|[A-Za-z0-9_]+)\s+from\s+)?['"]([^'"]+)['"]"#
);
static_regex!(
    re_sol_contract,
    r#"(?:^|\n)\s*(?:(abstract)\s+)?(contract|interface|library)\s+([A-Za-z0-9_]+)(?:\s+is\s+([A-Za-z0-9_,\s]+))?"#
);
static_regex!(
    re_sol_func,
    r#"(?:^|\n)\s*function\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)\s*(?:external|public|internal|private)?\s*(?:view|pure|payable)?(?:\s+(?:virtual|override(?:\([^)]*\))?))?"#
);
static_regex!(
    re_sol_modifier,
    r#"(?:^|\n)\s*modifier\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)"#
);
static_regex!(
    re_sol_call,
    r#"\b([A-Za-z0-9_]+)\s*\("#
);

static_regex!(
    re_php_use,
    r#"use\s+([A-Za-z0-9_\\]+)(?:\s+as\s+([A-Za-z0-9_]+))?\s*;"#
);
static_regex!(
    re_php_class,
    r#"(?:^|\n)\s*(?:(?:abstract|final|readonly)\s+)?(?:class|interface|trait|enum)\s+([A-Za-z0-9_]+)(?:\s+extends\s+([A-Za-z0-9_\\]+))?(?:\s+implements\s+([^{]+))?"#
);
static_regex!(
    re_php_func,
    r#"(?:^|\n)\s*(?:(?:public|protected|private|static|final|abstract|readonly)\s+)*function\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)(?:\s*:\s*([A-Za-z0-9_\\?|&]+))?"#
);
static_regex!(
    re_php_call,
    r#"(?:->|::|\b)([A-Za-z0-9_]+)\s*\("#
);
static_regex!(
    re_php_route,
    r#"#\[Route\(['"]([^'"]+)['"]|\$(?:app|router|route)->(?:get|post|put|delete|patch)\(['"]([^'"]+)['"]|Route::(?:get|post|put|delete|patch)\(['"]([^'"]+)['"]"#
);


static_regex!(re_md_link, r#"\[([^\]]+)\]\(([^)]+)\)"#);
static_regex!(
    re_md_wiki,
    r#"\[\[([^\]|#]+)(?:#[^\]|]*)?(?:\|[^\]]+)?\]\]"#
);
static_regex!(re_md_code, r#"`([A-Za-z0-9_]{3,40})`"#);

pub fn compute_complexity(slice: &str) -> u32 {
    let branches = re_complexity().find_iter(slice).count() as u32;
    1 + branches
}

#[derive(Debug, Clone)]
pub struct RawCall {
    pub caller: String,
    pub callee: String,
    pub relation: String,
    pub order_label: String,
}

pub struct FileAnalysis {
    pub rel_path: String,
    pub folder: String,
    pub raw_content: String,
    pub nodes: Vec<SymbolBubble>,
    pub calls: Vec<RawCall>,
    pub routes_declared: Vec<(String, String)>,
    pub routes_called: Vec<(String, String)>,
    pub ws_handlers: Vec<(String, String)>,
    pub ws_emits: Vec<(String, String)>,
}

#[derive(Default)]
pub struct ProjectScanner;

impl ProjectScanner {
    pub fn scan_project<P: AsRef<Path>>(&self, root: P) -> AetherGraph {
        self.scan_project_with_exclusions(root, &[])
    }

    pub fn scan_project_with_exclusions<P: AsRef<Path>>(
        &self,
        root: P,
        custom_exclusions: &[String],
    ) -> AetherGraph {
        let root_path = root.as_ref();
        let t0 = std::time::Instant::now();
        let project_name = root_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "AetherProject".to_string());

        let mut graph = AetherGraph::new(&project_name);
        graph.project_path = Some(root_path.to_string_lossy().to_string());

        let matches_wildcard = |pattern: &str, text: &str| -> bool {
            let tokens: Vec<&str> = pattern.split('*').collect();
            if tokens.len() == 1 {
                return pattern == text;
            }
            let mut rest = text;
            for (i, &token) in tokens.iter().enumerate() {
                if token.is_empty() {
                    continue;
                }
                if i == 0 {
                    if !rest.starts_with(token) {
                        return false;
                    }
                    rest = &rest[token.len()..];
                } else if i == tokens.len() - 1 {
                    return rest.ends_with(token);
                } else {
                    if let Some(pos) = rest.find(token) {
                        rest = &rest[pos + token.len()..];
                    } else {
                        return false;
                    }
                }
            }
            true
        };

        let is_excluded_custom = |path: &Path, rel_str: &str| -> bool {
            if custom_exclusions.is_empty() {
                return false;
            }
            let norm_rel = rel_str.replace('\\', "/").to_lowercase();
            let file_name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            for pattern in custom_exclusions {
                let pat = pattern.trim().trim_start_matches('/').trim_end_matches('/').to_lowercase();
                if pat.is_empty() {
                    continue;
                }
                // Match directory component: e.g. "tests", "test", "mocks"
                if norm_rel.split('/').any(|comp| comp == pat) {
                    return true;
                }
                // Prefix or contains match for folder paths: e.g. "tests" or "src/tests"
                if norm_rel.starts_with(&pat)
                    || norm_rel.contains(&format!("/{}/", pat))
                    || norm_rel.ends_with(&format!("/{}", pat))
                {
                    return true;
                }
                // Wildcard pattern matching on filename or relative path
                if pat.contains('*') {
                    if matches_wildcard(&pat, &file_name) || matches_wildcard(&pat, &norm_rel) {
                        return true;
                    }
                } else if file_name == pat || norm_rel == pat {
                    return true;
                }
            }
            false
        };

        // Detect if the target workspace is a Rust-centric project (root or workspace subcrates)
        let is_rust_project = root_path.join("Cargo.toml").exists()
            || root_path.join("Cargo.lock").exists()
            || root_path.join("client/Cargo.toml").exists()
            || root_path.join("common/Cargo.toml").exists()
            || root_path.join("eddie_hvnc/Cargo.toml").exists()
            || root_path.join("server/Cargo.toml").exists()
            || root_path.join("server/src-tauri/Cargo.toml").exists()
            || root_path.join("src-tauri/Cargo.toml").exists()
            || root_path.join("crates").is_dir();

        let exclude_dirs = [
            ".git",
            ".github",
            ".gitlab",
            ".aether",
            "node_modules",
            "target",
            "dist",
            "build",
            "out",
            ".next",
            ".nuxt",
            ".svelte-kit",
            ".turbo",
            ".vercel",
            "venv",
            ".venv",
            "env",
            "__pycache__",
            ".tox",
            ".mypy_cache",
            ".pytest_cache",
            ".ruff_cache",
            ".vscode",
            ".idea",
            ".vs",
            ".tauri_installer_assets_staging",
            "cosmoscope_output",
            "coverage",
            ".cache",
            "tmp",
            "temp",
            "vendor",
            "third_party",
            "external",
            "bower_components",
            "bin",
            "obj",
            ".cargo",
            ".rustup",
            "logs",
            "log",
            "site-packages",
            ".terraform",
            "pkg",
            "wasm",
            "debug",
            "release",
            "pytest",
            "benchmarks",
            "benches",
            "examples",
            "samples",
            "demos",
            "fixtures",
            "testdata",
            "test_data",
            "debug_scripts",
            "integration-tests",
            "test-loop-tests",
            "test-utils",
            "testlib",
            "test-contracts",
            "runtime-params-estimator",
            "jsonrpc-adversarial-primitives",
            "genesis-tools",
            "tools",
            "scripts",
            "fuzz",
            "nightly",
            // Embedded C/C++ toolchains, SDKs & static libraries (e.g. w64devkit / MinGW)
            "w64devkit",
            "mingw64",
            "mingw32",
            "msys64",
            "x86_64-w64-mingw32",
            "i686-w64-mingw32",
            "cmake",
            "llvm",
            "clang",
            "include",
            "libexec",
            "share",
            // Vendored / auxiliary non-Rust subprojects & static assets
            "libs",
            "client_apk",
            "client_c",
            "cpp_builder",
            "python_utils_testing",
            "stub",
            "bindings",
            "protos",
            "privileges_scripts",
            "res",
            "resources",
            "assets",
            "static",
            "public",
            "imgs",
            "images",
            "icons",
            "fonts",
            "country_flags",
            "media",
            "audio",
            "sounds",
            "lang",
            "locales",
            "i18n",
            "translations",
            "capabilities",
            "gen",
            "schemas",
            "docs",
            "doc",
        ];

        let mut candidate_files = Vec::new();
        for entry in WalkDir::new(root_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                let rel = e.path().strip_prefix(root_path).unwrap_or(e.path()).to_string_lossy();
                !name.starts_with('.')
                    && !exclude_dirs.iter().any(|d| name.eq_ignore_ascii_case(d))
                    && !name.contains("WebView2")
                    && !name.contains(".tmp")
                    && !is_excluded_custom(e.path(), &rel)
            })
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry.path().strip_prefix(root_path).unwrap_or(entry.path()).to_string_lossy();
            if is_excluded_custom(entry.path(), &rel) {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if meta.len() > 1_500_000 {
                    continue;
                }
            }
            let file_name = entry.file_name().to_string_lossy();
            let file_name_lower = file_name.to_lowercase();
            if file_name_lower.ends_with(".min.js")
                || file_name_lower.ends_with(".bundle.js")
                || file_name_lower.ends_with(".d.ts")
                || file_name_lower == "build.rs"
                || file_name_lower == "test.rs"
                || file_name_lower == "tests.rs"
                || file_name_lower.starts_with("temp_")
                || file_name_lower.starts_with("test_")
                || file_name_lower.ends_with("_test.rs")
                || file_name_lower.ends_with("_tests.rs")
                || file_name_lower.ends_with("_check.rs")
            {
                continue;
            }
            let ext = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            // For markdown files, only scan README.md architectural contracts
            if ext == "md" && file_name_lower != "readme.md" {
                continue;
            }
            // In Rust-centric projects, skip C/C++ toolchain/FFI files and Python scratch scripts
            if is_rust_project
                && matches!(
                    ext.as_str(),
                    "c" | "h" | "cpp" | "hpp" | "cc" | "hh" | "cxx" | "hxx" | "inl" | "py"
                )
            {
                continue;
            }
            if [
                "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "md", "cpp", "hpp", "cc", "hh",
                "cxx", "hxx", "c", "h", "inl", "sol", "yul", "php", "phtml", "php3", "php4", "php5", "php7", "phps",
            ]
            .contains(&ext.as_str())
            {
                candidate_files.push(entry.path().to_path_buf());
            }
        }

        let files_scanned = candidate_files.len();

        let results: Vec<FileAnalysis> = candidate_files
            .par_iter()
            .filter_map(|path| analyze_file(root_path, path))
            .collect();

        // Load aether.lock.toml if present to verify BLAKE3 contract hashes
        let mut locked_hashes: HashMap<String, String> = HashMap::new();
        let lock_path = root_path.join("aether.lock.toml");
        if let Ok(lock_str) = fs::read_to_string(&lock_path) {
            if let Ok(manifest) = toml::from_str::<ProjectManifest>(&lock_str) {
                for (gal_key, gal_cfg) in manifest.galaxies {
                    if let Some(h) = gal_cfg.readme_blake3 {
                        let norm_path = gal_cfg
                            .path
                            .trim_start_matches("./")
                            .trim_start_matches('/')
                            .replace('\\', "/");
                        locked_hashes.insert(norm_path, h.clone());
                        locked_hashes.insert(gal_key, h);
                    }
                }
            }
        }

        // Parse README.md contracts across all folders
        let mut contracts_by_folder: HashMap<String, ReadmeContract> = HashMap::new();
        for res in &results {
            if res.rel_path.to_lowercase().ends_with("readme.md") {
                if let Ok(contract) = ReadmeContract::parse(&res.folder, &res.raw_content) {
                    contracts_by_folder.insert(res.folder.clone(), contract);
                }
            }
        }

        let mut all_raw_calls = Vec::new();
        let mut routes_declared = Vec::new();
        let mut routes_called = Vec::new();
        let mut ws_handlers = Vec::new();
        let mut ws_emits = Vec::new();
        let mut file_contents: Vec<(String, String, String)> = Vec::new();

        for res in results {
            file_contents.push((res.rel_path, res.folder, res.raw_content));
            graph.nodes.extend(res.nodes);
            all_raw_calls.extend(res.calls);
            routes_declared.extend(res.routes_declared);
            routes_called.extend(res.routes_called);
            ws_handlers.extend(res.ws_handlers);
            ws_emits.extend(res.ws_emits);
        }

        // Resolve AST edges (contains, imports, calls, renders, inherits, network, submits, documents)
        let resolved_edges = resolve_graph_edges(&mut graph.nodes, &all_raw_calls);
        graph.edges.extend(resolved_edges);

        // Attach BLAKE3 / EARS contract metadata and run SecurityLinter on all scanned files
        let find_contract_for_folder = |folder: &str| -> Option<&ReadmeContract> {
            if let Some(c) = contracts_by_folder.get(folder) {
                return Some(c);
            }
            let mut cur = folder;
            while let Some(idx) = cur.rfind('/') {
                cur = &cur[..idx];
                if let Some(c) = contracts_by_folder.get(cur) {
                    return Some(c);
                }
            }
            contracts_by_folder.get("root")
        };

        // Enrich all nodes with their folder's BLAKE3 hash, lock status & EARS count
        let mut tamper_edges: Vec<AffiliationEdge> = Vec::new();
        for node in &mut graph.nodes {
            if let Some(contract) = find_contract_for_folder(&node.folder) {
                node.metadata.blake3_hash = Some(contract.blake3_hash.clone());
                node.metadata.ears_count = contract.ears_requirements.len();
                let expected = locked_hashes
                    .get(&contract.folder_path)
                    .or_else(|| locked_hashes.get(&node.folder))
                    .or_else(|| locked_hashes.get(&node.galaxy));
                if let Some(exp_hash) = expected {
                    let verified = contract.verify_lock(exp_hash);
                    node.metadata.lock_verified = Some(verified);
                    if !verified && node.file_path.to_lowercase().ends_with("readme.md") {
                        node.has_security_violation = true;
                        let msg = format!(
                            "[BLAKE3_TAMPERED] Contrat {} modifié sans signature dans aether.lock.toml (attendu: {}…, actuel: {}…)",
                            node.file_path,
                            &exp_hash[..exp_hash.len().min(8)],
                            &contract.blake3_hash[..contract.blake3_hash.len().min(8)]
                        );
                        let anomaly = CveAnomalyRecord {
                            rule_id: "BLAKE3_TAMPERED".to_string(),
                            cve_id: "CWE-353 / BLAKE3-INTEGRITY".to_string(),
                            severity: "CRITICAL".to_string(),
                            line_number: 1,
                            file_path: node.file_path.clone(),
                            message: msg.clone(),
                            transmitted_data: Some("contract_readme_hash".to_string()),
                            transmitted_type: Some("BLAKE3 (Tampered Hash Mismatch)".to_string()),
                            remediation: "Re-sign contract with AetherScope or revert unauthorized modifications.".to_string(),
                        };
                        node.metadata.violations.push(msg.clone());
                        node.metadata.cve_anomalies.push(anomaly);
                        tamper_edges.push(AffiliationEdge::new_cve_violation(
                            node.id.clone(),
                            node.id.clone(),
                            &msg,
                            Some("CWE-353 / BLAKE3-INTEGRITY".to_string()),
                            Some("CRITICAL".to_string()),
                            Some("contract_readme_hash".to_string()),
                            Some("BLAKE3 (Tampered Hash Mismatch)".to_string()),
                        ));
                    }
                }
            }
        }
        graph.edges.extend(tamper_edges);

        // Pre-index node indices by file_path and by node ID for instant O(1) lookup during violation mapping
        let mut nodes_by_file: HashMap<String, Vec<usize>> = HashMap::new();
        let mut node_idx_by_id: HashMap<String, usize> = HashMap::with_capacity(graph.nodes.len());
        for (idx, n) in graph.nodes.iter().enumerate() {
            nodes_by_file.entry(n.file_path.clone()).or_default().push(idx);
            node_idx_by_id.insert(n.id.clone(), idx);
        }

        // Execute SecurityLinter::audit_file in parallel via Rayon across all non-markdown files
        let parallel_violations: Vec<(String, Option<String>, Vec<security_linter::linter::LinterViolation>)> = file_contents
            .par_iter()
            .filter_map(|(rel_path, folder, raw_content)| {
                if rel_path.to_lowercase().ends_with(".md") {
                    return None;
                }
                let contract = find_contract_for_folder(folder);
                let violations = SecurityLinter::audit_file(rel_path, raw_content, contract);
                if violations.is_empty() {
                    None
                } else {
                    let contract_folder = contract.map(|c| c.folder_path.clone());
                    Some((rel_path.clone(), contract_folder, violations))
                }
            })
            .collect();

        let mut linter_edges: Vec<AffiliationEdge> = Vec::new();
        for (rel_path, contract_folder, violations) in parallel_violations {
            let file_node_id = format!("file::{}", rel_path);
            let readme_target_id = contract_folder
                .map(|cf| {
                    if cf == "root" {
                        "file::README.md".to_string()
                    } else {
                        format!("file::{}/README.md", cf)
                    }
                })
                .unwrap_or_else(|| file_node_id.clone());

            let file_node_indices = nodes_by_file.get(&rel_path);
            let has_readme_target = node_idx_by_id.contains_key(&readme_target_id);

            for v in violations {
                let formatted = format!(
                    "[{}] ({}) L{}: {} — Fix: {}",
                    v.rule_id, v.severity, v.line_number, v.message, v.remediation
                );

                let anomaly_rec = CveAnomalyRecord {
                    rule_id: v.rule_id.clone(),
                    cve_id: v.cve_id.clone(),
                    severity: v.severity.clone(),
                    line_number: v.line_number,
                    file_path: v.file_path.clone(),
                    message: v.message.clone(),
                    transmitted_data: v.transmitted_data.clone(),
                    transmitted_type: v.transmitted_type.clone(),
                    remediation: v.remediation.clone(),
                };

                // Mark file node and closest symbol node in that file (RULE_MAX_LINES is strictly file-level, never attached to a symbol)
                let is_file_level_rule = v.rule_id == "RULE_MAX_LINES";
                let mut best_sym_id: Option<String> = None;
                let mut best_line = 0usize;

                if let Some(indices) = file_node_indices {
                    for &idx in indices {
                        let n = &mut graph.nodes[idx];
                        if n.is_file {
                            n.has_security_violation = true;
                            n.metadata.violations.push(formatted.clone());
                        } else if !is_file_level_rule && n.line_number <= v.line_number && n.line_number >= best_line {
                            best_line = n.line_number;
                            best_sym_id = Some(n.id.clone());
                        }
                    }
                }

                if !is_file_level_rule && best_sym_id.is_some() {
                    let sym_id = best_sym_id.as_ref().unwrap();
                    if let Some(&sym_idx) = node_idx_by_id.get(sym_id) {
                        let sym_node = &mut graph.nodes[sym_idx];
                        sym_node.has_security_violation = true;
                        sym_node.metadata.violations.push(formatted.clone());
                        sym_node.metadata.cve_anomalies.push(anomaly_rec.clone());
                    }
                } else if let Some(&file_idx) = node_idx_by_id.get(&file_node_id) {
                    graph.nodes[file_idx].metadata.cve_anomalies.push(anomaly_rec.clone());
                }

                let src_id = if is_file_level_rule {
                    file_node_id.clone()
                } else {
                    best_sym_id.unwrap_or_else(|| file_node_id.clone())
                };
                let tgt_id = if has_readme_target {
                    readme_target_id.clone()
                } else {
                    file_node_id.clone()
                };
                linter_edges.push(AffiliationEdge::new_cve_violation(
                    src_id,
                    tgt_id,
                    &format!("{}: {}", v.rule_id, v.message),
                    Some(v.cve_id.clone()),
                    Some(v.severity.clone()),
                    v.transmitted_data.clone(),
                    v.transmitted_type.clone(),
                ));
            }
        }
        graph.edges.extend(linter_edges);

        // Cross-Boundary Filter 2 : NetworkRPC Matcher (Client -> Server Laser Orange)
        for (call_route, client_id) in &routes_called {
            let norm_call = call_route.to_lowercase().replace('/', "_");
            for (route_name, server_id) in &routes_declared {
                if server_id.to_lowercase().contains(&norm_call)
                    || route_name.to_lowercase().contains(&norm_call)
                {
                    graph.add_edge(AffiliationEdge::new_rpc(
                        client_id.clone(),
                        server_id.clone(),
                        call_route,
                    ));
                }
            }
        }

        // Cross-Boundary Filter 3 : RealtimeWS Matcher (Client -> Server Laser Cyan)
        for (pkt_name, client_id) in &ws_emits {
            let norm_pkt = pkt_name.to_lowercase();
            for (handler_name, server_id) in &ws_handlers {
                if handler_name.to_lowercase().contains(&norm_pkt)
                    || server_id.to_lowercase().contains(&norm_pkt)
                {
                    graph.add_edge(AffiliationEdge::new_realtime(
                        client_id.clone(),
                        server_id.clone(),
                        pkt_name,
                    ));
                }
            }
        }

        // Compute the 4 mathematical layouts (Obsidian, System, Flow, Classes)
        compute_all_layouts(&mut graph.nodes);

        // Compute stats (degrees, dead code, layers, relations, sides)
        graph.stats.files_scanned = files_scanned;
        graph.compute_stats();
        graph.stats.elapsed_ms = t0.elapsed().as_millis();

        graph
    }
}

pub fn detect_galaxy(rel_path: &str) -> String {
    let fp = rel_path.replace('\\', "/").to_lowercase();
    if fp.starts_with("contracts") || fp.starts_with("shared") || fp.contains("/contracts/") {
        "contracts".to_string()
    } else if fp.starts_with("client") || fp.starts_with("frontend") || fp.starts_with("web") {
        "client".to_string()
    } else if fp.starts_with("server") || fp.starts_with("backend") {
        "server".to_string()
    } else if fp.starts_with("data-nodes") || fp.contains("/db/") || fp.contains("/database/") {
        "data-nodes".to_string()
    } else {
        "core".to_string()
    }
}

pub fn detect_layer(file_path: &str, name: &str) -> String {
    let fp = file_path.to_lowercase().replace('\\', "/");
    let nn = name.to_lowercase();

    if fp.ends_with(".md")
        || fp.contains("docs")
        || fp.contains("documentation")
        || nn.contains("readme")
    {
        "docs".to_string()
    } else if fp.contains("ui")
        || fp.contains("views")
        || fp.contains("components")
        || fp.contains("frontend")
        || fp.contains("dashboard")
        || fp.contains("renderer")
        || fp.contains("desktop-shell")
        || nn.contains("view")
        || nn.contains("page")
        || nn.contains("widget")
        || nn.contains("panel")
        || nn.contains("render")
        || nn.contains("card")
        || nn.contains("modal")
        || nn.contains("button")
    {
        "ui".to_string()
    } else if fp.contains("controllers")
        || fp.contains("handlers")
        || fp.contains("routes")
        || fp.contains("api")
        || fp.contains("endpoints")
        || fp.contains("cli")
        || fp.contains("actions")
        || fp.contains("hooks")
        || fp.contains("transport")
        || nn.contains("controller")
        || nn.contains("handler")
        || nn.contains("route")
        || nn.contains("command")
        || nn.contains("action")
    {
        "action".to_string()
    } else if fp.contains("models")
        || fp.contains("entities")
        || fp.contains("database")
        || fp.contains("db")
        || fp.contains("schemas")
        || fp.contains("repositories")
        || fp.contains("stores")
        || fp.contains("data-nodes")
        || nn.contains("model")
        || nn.contains("entity")
        || nn.contains("schema")
        || nn.contains("table")
        || nn.contains("store")
    {
        "database".to_string()
    } else if fp.contains("config")
        || fp.contains("settings")
        || fp.contains("env")
        || fp.contains("manifest")
        || nn.contains("config")
        || nn.contains("settings")
    {
        "config".to_string()
    } else if fp.contains("utils")
        || fp.contains("helpers")
        || fp.contains("tools")
        || fp.contains("common")
        || nn.contains("util")
        || nn.contains("helper")
        || nn.contains("format")
    {
        "utility".to_string()
    } else {
        "domain".to_string()
    }
}

pub fn detect_stereotype(name: &str, is_class: bool, layer: &str, bases: &[String]) -> String {
    let nn = name.to_lowercase();
    if nn.contains("form") || bases.iter().any(|b| b.to_lowercase().contains("form")) {
        return "«Form»".to_string();
    }
    if is_class {
        if nn.contains("model")
            || bases.iter().any(|b| b.to_lowercase().contains("model"))
            || layer == "database"
        {
            "«Model»".to_string()
        } else if nn.contains("service")
            || bases.iter().any(|b| b.to_lowercase().contains("service"))
        {
            "«Service»".to_string()
        } else if nn.contains("controller") || nn.contains("handler") {
            "«Controller»".to_string()
        } else if nn.contains("view") || nn.contains("component") || layer == "ui" {
            "«Component»".to_string()
        } else if nn.contains("config") || nn.contains("manifest") {
            "«Config»".to_string()
        } else if nn.contains("helper") || nn.contains("util") {
            "«Utility»".to_string()
        } else {
            "«Class»".to_string()
        }
    } else if layer == "action" {
        "«Action»".to_string()
    } else if layer == "ui" {
        "«Component»".to_string()
    } else {
        "«Function»".to_string()
    }
}

fn folder_from_rel_path(rel_path: &str) -> String {
    Path::new(rel_path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "root".to_string())
}

fn extract_leading_doc(content: &str, start_byte: usize, is_rust: bool) -> String {
    let end = start_byte.min(content.len());
    if end == 0 {
        return String::new();
    }
    // Only inspect up to the last 2048 bytes before start_byte to avoid O(N) whole-file allocations
    let mut win_start = end.saturating_sub(2048);
    while win_start < end && !content.is_char_boundary(win_start) {
        win_start += 1;
    }
    let prefix = &content[win_start..end];

    let mut collected = Vec::new();
    let mut in_block_comment = false;
    let mut skipped_blanks = 0;

    for raw_line in prefix.lines().rev() {
        let line = raw_line.trim();
        if line.is_empty() {
            if collected.is_empty() && skipped_blanks < 2 {
                skipped_blanks += 1;
                continue;
            } else {
                break;
            }
        }

        // Rust attributes: #[...], #![...]
        if is_rust && (line.starts_with("#[") || line.starts_with("#![") || line == "]") {
            continue;
        }
        // JS decorators: @...
        if !is_rust
            && line.starts_with('@')
            && !line.starts_with("@param")
            && !line.starts_with("@return")
            && !line.starts_with("@type")
        {
            continue;
        }

        if in_block_comment {
            if let Some(pos) = line.find("/*") {
                let part = line[pos + 2..].trim_start_matches('*').trim();
                if !part.is_empty() {
                    collected.push(part.to_string());
                }
                in_block_comment = false;
            } else {
                let part = line.trim_start_matches('*').trim();
                collected.push(part.to_string());
            }
            continue;
        }

        if line.ends_with("*/") {
            in_block_comment = true;
            if let Some(pos) = line.find("/*") {
                let inner = &line[pos + 2..line.len() - 2];
                let part = inner.trim_start_matches('*').trim();
                if !part.is_empty() {
                    collected.push(part.to_string());
                }
                in_block_comment = false;
            } else {
                let inner = &line[..line.len() - 2];
                let part = inner.trim_start_matches('*').trim();
                if !part.is_empty() {
                    collected.push(part.to_string());
                }
            }
            continue;
        }

        // Line comments:
        if line.starts_with("///") || line.starts_with("//!") {
            let text = line[3..].trim();
            collected.push(text.to_string());
        } else if line.starts_with("//") {
            let text = line[2..].trim();
            collected.push(text.to_string());
        } else if line.starts_with('#') {
            let text = line[1..].trim();
            collected.push(text.to_string());
        } else {
            break;
        }
    }

    collected.reverse();
    collected.join("\n")
}

fn extract_python_docstring(body_slice: &str) -> String {
    let trimmed = body_slice.trim_start();
    if trimmed.starts_with("\"\"\"") {
        if let Some(end) = trimmed[3..].find("\"\"\"") {
            return trimmed[3..3 + end].trim().to_string();
        }
    } else if trimmed.starts_with("'''") {
        if let Some(end) = trimmed[3..].find("'''") {
            return trimmed[3..3 + end].trim().to_string();
        }
    }
    String::new()
}

fn analyze_file(root_path: &Path, file_path: &Path) -> Option<FileAnalysis> {
    let content = fs::read_to_string(file_path).ok()?;
    let rel_path = file_path
        .strip_prefix(root_path)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");

    let ext = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => Some(analyze_jsts(&rel_path, &content)),
        "py" => Some(analyze_python(&rel_path, &content)),
        "rs" => Some(analyze_rust(&rel_path, &content)),
        "cpp" | "hpp" | "cc" | "hh" | "cxx" | "hxx" | "c" | "h" | "inl" => {
            Some(analyze_cpp(&rel_path, &content))
        }
        "sol" | "yul" => Some(analyze_solidity(&rel_path, &content)),
        "md" => Some(analyze_markdown(&rel_path, &content)),
        "php" | "phtml" | "php3" | "php4" | "php5" | "php7" | "phps" => {
            Some(analyze_php(&rel_path, &content))
        }
        _ => None,
    }
}

fn analyze_php(rel_path: &str, content: &str) -> FileAnalysis {
    let mut nodes = Vec::new();
    let mut calls = Vec::new();
    let mut routes_declared = Vec::new();
    let routes_called = Vec::new();
    let ws_handlers = Vec::new();
    let ws_emits = Vec::new();

    let galaxy = detect_galaxy(rel_path);
    let file_node_id = format!("file::{}", rel_path);
    let file_layer = detect_layer(rel_path, "");
    let file_name = Path::new(rel_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());
    let folder = folder_from_rel_path(rel_path);

    let mut file_bubble = SymbolBubble::new(
        file_node_id.clone(),
        file_name,
        rel_path.to_string(),
        1,
        SymbolKind::File,
        galaxy.clone(),
        file_layer,
    );
    file_bubble.folder = folder.clone();
    file_bubble.docstring = format!("Module / Script PHP {}", rel_path);
    file_bubble.lines_count = content.lines().count();
    file_bubble.complexity = compute_complexity(content);
    file_bubble.metadata.lines = file_bubble.lines_count;
    file_bubble.metadata.file_type = "php".to_string();
    nodes.push(file_bubble);

    // 1. PHP use imports
    for cap in re_php_use().captures_iter(content) {
        if let Some(target) = cap.get(1) {
            let full_import = target.as_str().trim();
            let short_name = cap
                .get(2)
                .map(|m| m.as_str().trim())
                .unwrap_or_else(|| full_import.rsplit('\\').next().unwrap_or(full_import));
            calls.push(RawCall {
                caller: file_node_id.clone(),
                callee: short_name.to_string(),
                relation: "imports".to_string(),
                order_label: format!("Import {}", full_import),
            });
        }
    }

    // 2. PHP Classes / Interfaces / Traits
    for cap in re_php_class().captures_iter(content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(content, match_0.start());
        let extends_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let implements_str = cap.get(3).map(|m| m.as_str()).unwrap_or("");

        let mut bases: Vec<String> = Vec::new();
        if !extends_str.is_empty() {
            let clean = extends_str.rsplit('\\').next().unwrap_or(extends_str).trim();
            if !clean.is_empty() {
                bases.push(clean.to_string());
            }
        }
        if !implements_str.is_empty() {
            for imp in implements_str.split(',') {
                let clean = imp.trim().rsplit('\\').next().unwrap_or(imp).trim();
                if !clean.is_empty() {
                    bases.push(clean.to_string());
                }
            }
        }

        let node_id = format!("{}::{}", rel_path, name);
        let layer = detect_layer(rel_path, name);
        let stereo = detect_stereotype(name, true, &layer, &bases);

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        for b in &bases {
            calls.push(RawCall {
                caller: node_id.clone(),
                callee: b.clone(),
                relation: "inherits".to_string(),
                order_label: format!("Hérite de {}", b),
            });
        }

        let mut class_bubble = SymbolBubble::new(
            node_id.clone(),
            name.to_string(),
            rel_path.to_string(),
            line_no,
            SymbolKind::ClassOrStruct,
            galaxy.clone(),
            layer,
        );
        class_bubble.folder = folder.clone();
        class_bubble.stereotype = stereo;
        let class_doc = extract_leading_doc(content, match_0.start(), false);
        class_bubble.docstring = class_doc;
        class_bubble.complexity = 1;
        nodes.push(class_bubble);
    }

    // 3. PHP Functions & Methods
    for cap in re_php_func().captures_iter(content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(content, match_0.start());
        let args_str = cap.get(2).map(|m| m.as_str().trim().to_string());
        let ret_str = cap.get(3).map(|m| m.as_str().trim().to_string());

        let node_id = format!("{}::{}", rel_path, name);
        let layer = detect_layer(rel_path, name);
        let stereo = detect_stereotype(name, false, &layer, &[]);

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        let body_slice = safe_slice(content, match_0.end(), 2000);
        for c_cap in re_php_call().captures_iter(body_slice) {
            let callee = c_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !callee.is_empty() && callee != name && callee != "function" && callee != "echo" && callee != "print" {
                calls.push(RawCall {
                    caller: node_id.clone(),
                    callee: callee.to_string(),
                    relation: "calls".to_string(),
                    order_label: format!("Appel {}", callee),
                });
            }
        }

        let mut fn_bubble = SymbolBubble::new(
            node_id,
            name.to_string(),
            rel_path.to_string(),
            line_no,
            SymbolKind::Function,
            galaxy.clone(),
            layer,
        );
        fn_bubble.folder = folder.clone();
        fn_bubble.stereotype = stereo;
        fn_bubble.complexity = compute_complexity(body_slice);
        let fn_doc = extract_leading_doc(content, match_0.start(), false);
        fn_bubble.docstring = fn_doc.clone();
        fn_bubble.metadata.methods.push(AstMethod {
            name: name.to_string(),
            args: args_str,
            returns: ret_str,
            is_async: None,
            calls: Vec::new(),
            docstring: if fn_doc.is_empty() { None } else { Some(fn_doc) },
        });
        nodes.push(fn_bubble);
    }

    // 4. PHP Routes
    for cap in re_php_route().captures_iter(content) {
        if let Some(r_path) = cap.get(1).or_else(|| cap.get(2)).or_else(|| cap.get(3)) {
            let p_str = r_path.as_str().trim();
            if !p_str.is_empty() {
                routes_declared.push((p_str.to_string(), file_node_id.clone()));
            }
        }
    }

    FileAnalysis {
        rel_path: rel_path.to_string(),
        folder,
        raw_content: content.to_string(),
        nodes,
        calls,
        routes_declared,
        routes_called,
        ws_handlers,
        ws_emits,
    }
}

fn analyze_jsts(rel_path: &str, content: &str) -> FileAnalysis {
    let mut nodes = Vec::new();
    let mut calls = Vec::new();
    let mut routes_declared = Vec::new();
    let mut routes_called = Vec::new();
    let ws_handlers = Vec::new();
    let mut ws_emits = Vec::new();

    let is_react = rel_path.ends_with(".jsx")
        || rel_path.ends_with(".tsx")
        || content.contains("React")
        || content.contains("useState");

    let galaxy = detect_galaxy(rel_path);
    let file_node_id = format!("file::{}", rel_path);
    let file_layer = detect_layer(rel_path, "");
    let file_name = Path::new(rel_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());
    let folder = folder_from_rel_path(rel_path);

    let mut file_bubble = SymbolBubble::new(
        file_node_id.clone(),
        file_name,
        rel_path.to_string(),
        1,
        SymbolKind::File,
        galaxy.clone(),
        file_layer,
    );
    file_bubble.folder = folder.clone();
    file_bubble.docstring = format!("Fichier source {}", rel_path);
    file_bubble.lines_count = content.lines().count();
    file_bubble.complexity = compute_complexity(content);
    file_bubble.metadata.lines = file_bubble.lines_count;
    file_bubble.metadata.file_type = if is_react {
        "react".to_string()
    } else {
        "javascript".to_string()
    };
    nodes.push(file_bubble);

    let cur_dir = Path::new(rel_path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();

    // 1. ES6 Imports
    for cap in re_js_import().captures_iter(content) {
        if let Some(target_match) = cap.get(4) {
            let target_path = target_match.as_str();
            if target_path.starts_with('.') {
                let joined = if cur_dir.is_empty() {
                    target_path.to_string()
                } else {
                    format!("{}/{}", cur_dir, target_path)
                };
                let norm = normalize_path(&joined);
                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: norm,
                    relation: "imports".to_string(),
                    order_label: "Import".to_string(),
                });
            } else {
                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: target_path.to_string(),
                    relation: "imports".to_string(),
                    order_label: format!("Import {}", target_path),
                });
            }

            if let Some(named_match) = cap.get(1) {
                let inner = named_match.as_str().trim_matches(|c| c == '{' || c == '}');
                for sym in inner.split(',') {
                    let clean = sym.split(" as ").next().unwrap_or("").trim();
                    if !clean.is_empty() {
                        calls.push(RawCall {
                            caller: file_node_id.clone(),
                            callee: clean.to_string(),
                            relation: "imports".to_string(),
                            order_label: format!("Import {}", clean),
                        });
                    }
                }
            }
        }
    }

    // CommonJS require('...')
    for cap in re_js_require().captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let target_path = m.as_str();
            if target_path.starts_with('.') {
                let joined = if cur_dir.is_empty() {
                    target_path.to_string()
                } else {
                    format!("{}/{}", cur_dir, target_path)
                };
                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: normalize_path(&joined),
                    relation: "imports".to_string(),
                    order_label: "Require".to_string(),
                });
            }
        }
    }

    // 2. Classes & Interfaces
    for cap in re_js_class().captures_iter(content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let extends = cap.get(2).map(|m| m.as_str().to_string());
        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(content, match_0.start());
        let class_slice = safe_slice(content, match_0.end(), 2500);

        let node_id = format!("{}::{}", rel_path, name);
        let layer = detect_layer(rel_path, name);
        let bases = extends.clone().map(|e| vec![e]).unwrap_or_default();
        let stereo = detect_stereotype(name, true, &layer, &bases);

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        if let Some(ref ext) = extends {
            calls.push(RawCall {
                caller: node_id.clone(),
                callee: ext.clone(),
                relation: "inherits".to_string(),
                order_label: format!("Hérite de {}", ext),
            });
        }

        let mut class_bubble = SymbolBubble::new(
            node_id,
            name.to_string(),
            rel_path.to_string(),
            line_no,
            SymbolKind::ClassOrStruct,
            galaxy.clone(),
            layer,
        );
        class_bubble.folder = folder.clone();
        class_bubble.stereotype = stereo;
        class_bubble.complexity = compute_complexity(class_slice);
        class_bubble.docstring = extract_leading_doc(content, match_0.start(), false);
        nodes.push(class_bubble);
    }

    // 3. Functions & React Components
    let mut seen_fn_ids = HashSet::new();

    for cap in re_js_func()
        .captures_iter(content)
        .chain(re_js_arrow().captures_iter(content))
    {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let node_id = format!("{}::{}", rel_path, name);
        if !seen_fn_ids.insert(node_id.clone()) {
            continue;
        }

        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(content, match_0.start());
        let body_slice = safe_slice(content, match_0.end(), 6000);

        let is_comp =
            name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && is_react;
        let is_form = name.to_lowercase().contains("form")
            || body_slice.contains("<form")
            || body_slice.contains("onSubmit")
            || body_slice.contains("handleSubmit")
            || body_slice.contains("useForm");

        let layer = if is_comp {
            "ui".to_string()
        } else {
            detect_layer(rel_path, name)
        };
        let stereo = if is_form {
            "«Form»".to_string()
        } else if is_comp {
            "«Component»".to_string()
        } else {
            detect_stereotype(name, false, &layer, &[])
        };

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        for c_cap in re_call_ident().captures_iter(body_slice) {
            let callee = c_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !callee.is_empty() && callee != name && !is_js_keyword(callee) {
                calls.push(RawCall {
                    caller: node_id.clone(),
                    callee: callee.to_string(),
                    relation: "calls".to_string(),
                    order_label: format!("Appel {}", callee),
                });
            }
        }

        for j_cap in re_jsx_tag().captures_iter(body_slice) {
            let rendered = j_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !rendered.is_empty() && rendered != name {
                calls.push(RawCall {
                    caller: node_id.clone(),
                    callee: rendered.to_string(),
                    relation: "renders".to_string(),
                    order_label: format!("Rend <{}/>", rendered),
                });
            }
        }

        for net_cap in re_js_network().captures_iter(body_slice) {
            if let Some(endpoint) = net_cap.get(1) {
                let ep = endpoint.as_str().trim();
                if !ep.is_empty() {
                    calls.push(RawCall {
                        caller: node_id.clone(),
                        callee: ep.to_string(),
                        relation: "network".to_string(),
                        order_label: format!("Réseau {}", ep),
                    });
                    routes_called.push((ep.to_string(), node_id.clone()));
                }
            }
        }

        for sub_cap in re_js_submit().captures_iter(body_slice) {
            if let Some(handler) = sub_cap.get(1) {
                let h_name = handler.as_str();
                if h_name != name {
                    calls.push(RawCall {
                        caller: node_id.clone(),
                        callee: h_name.to_string(),
                        relation: "submits".to_string(),
                        order_label: format!("Soumet {}", h_name),
                    });
                }
            }
        }

        for ws_cap in re_js_ws_emit().captures_iter(body_slice) {
            if let Some(pkt) = ws_cap.get(1) {
                ws_emits.push((pkt.as_str().to_string(), node_id.clone()));
            }
        }

        if galaxy == "server" && (name.to_lowercase().contains("handle") || name.to_lowercase().contains("route") || rel_path.contains("api")) {
            routes_declared.push((name.to_string(), node_id.clone()));
        }

        let mut comp_attrs = Vec::new();
        for h_cap in re_js_hook().captures_iter(body_slice) {
            if let Some(h) = h_cap.get(1) {
                let h_name = h.as_str();
                if !comp_attrs.iter().any(|a: &AstAttribute| a.name == h_name) {
                    comp_attrs.push(AstAttribute {
                        name: h_name.to_string(),
                        r#type: Some("Hook".to_string()),
                    });
                }
            }
        }

        let kind = if is_form {
            SymbolKind::FormOrEndpoint
        } else {
            SymbolKind::Function
        };
        let mut fn_bubble = SymbolBubble::new(
            node_id,
            name.to_string(),
            rel_path.to_string(),
            line_no,
            kind,
            galaxy.clone(),
            layer,
        );
        fn_bubble.folder = folder.clone();
        fn_bubble.stereotype = stereo;
        fn_bubble.complexity = compute_complexity(body_slice);
        let fn_doc = extract_leading_doc(content, match_0.start(), false);
        fn_bubble.docstring = fn_doc.clone();
        fn_bubble.symbol_type = if is_form {
            "form".to_string()
        } else if is_comp {
            "component".to_string()
        } else {
            "function".to_string()
        };
        fn_bubble.metadata.attributes = comp_attrs;
        fn_bubble.metadata.methods.push(AstMethod {
            name: name.to_string(),
            args: None,
            returns: None,
            is_async: Some(match_0.as_str().contains("async")),
            calls: Vec::new(),
            docstring: if fn_doc.is_empty() { None } else { Some(fn_doc) },
        });
        nodes.push(fn_bubble);
    }

    FileAnalysis {
        rel_path: rel_path.to_string(),
        folder,
        raw_content: content.to_string(),
        nodes,
        calls,
        routes_declared,
        routes_called,
        ws_handlers,
        ws_emits,
    }
}

fn analyze_python(rel_path: &str, content: &str) -> FileAnalysis {
    let mut nodes = Vec::new();
    let mut calls = Vec::new();
    let mut routes_declared = Vec::new();
    let mut routes_called = Vec::new();
    let ws_handlers = Vec::new();
    let ws_emits = Vec::new();

    let galaxy = detect_galaxy(rel_path);
    let file_node_id = format!("file::{}", rel_path);
    let file_layer = detect_layer(rel_path, "");
    let file_name = Path::new(rel_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());
    let folder = folder_from_rel_path(rel_path);

    let mut file_bubble = SymbolBubble::new(
        file_node_id.clone(),
        file_name,
        rel_path.to_string(),
        1,
        SymbolKind::File,
        galaxy.clone(),
        file_layer,
    );
    file_bubble.folder = folder.clone();
    file_bubble.docstring = format!("Module Python {}", rel_path);
    file_bubble.lines_count = content.lines().count();
    file_bubble.complexity = compute_complexity(content);
    file_bubble.metadata.lines = file_bubble.lines_count;
    file_bubble.metadata.file_type = "python".to_string();
    nodes.push(file_bubble);

    // 1. Python Imports
    for cap in re_py_import().captures_iter(content) {
        if let Some(from_mod) = cap.get(1) {
            let mod_name = from_mod.as_str().replace('.', "/");
            calls.push(RawCall {
                caller: file_node_id.clone(),
                callee: mod_name.clone(),
                relation: "imports".to_string(),
                order_label: format!("Import {}", mod_name),
            });
            if let Some(imported_syms) = cap.get(2) {
                for sym in imported_syms.as_str().split(',') {
                    let clean = sym.trim();
                    if !clean.is_empty() && clean != "*" {
                        calls.push(RawCall {
                            caller: file_node_id.clone(),
                            callee: clean.to_string(),
                            relation: "imports".to_string(),
                            order_label: format!("Import {}", clean),
                        });
                    }
                }
            }
        } else if let Some(direct_imp) = cap.get(3) {
            for imp in direct_imp.as_str().split(',') {
                let clean = imp.split(" as ").next().unwrap_or("").trim();
                if !clean.is_empty() {
                    calls.push(RawCall {
                        caller: file_node_id.clone(),
                        callee: clean.to_string(),
                        relation: "imports".to_string(),
                        order_label: format!("Import {}", clean),
                    });
                }
            }
        }
    }

    // 2. Classes
    for cap in re_py_class().captures_iter(content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(content, match_0.start());
        let bases_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let bases: Vec<String> = bases_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "object")
            .collect();

        let node_id = format!("{}::{}", rel_path, name);
        let layer = detect_layer(rel_path, name);
        let stereo = detect_stereotype(name, true, &layer, &bases);

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        for b in &bases {
            calls.push(RawCall {
                caller: node_id.clone(),
                callee: b.clone(),
                relation: "inherits".to_string(),
                order_label: format!("Hérite de {}", b),
            });
        }

        let body_slice = safe_slice(content, match_0.end(), 1500);
        let mut attributes = Vec::new();
        for f_cap in re_py_field().captures_iter(body_slice) {
            let f_name = f_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let f_type = f_cap.get(2).map(|m| m.as_str().trim().to_string());
            if !f_name.is_empty() && f_name != "def" && f_name != "class" {
                attributes.push(AstAttribute {
                    name: f_name.to_string(),
                    r#type: f_type,
                });
            }
        }

        let mut class_bubble = SymbolBubble::new(
            node_id,
            name.to_string(),
            rel_path.to_string(),
            line_no,
            SymbolKind::ClassOrStruct,
            galaxy.clone(),
            layer,
        );
        class_bubble.folder = folder.clone();
        class_bubble.stereotype = stereo;
        class_bubble.complexity = compute_complexity(body_slice);
        class_bubble.metadata.attributes = attributes;
        let py_class_doc = extract_python_docstring(body_slice);
        class_bubble.docstring = if !py_class_doc.is_empty() {
            py_class_doc
        } else {
            extract_leading_doc(content, match_0.start(), false)
        };
        nodes.push(class_bubble);
    }

    // 3. Functions
    for cap in re_py_func().captures_iter(content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if name.is_empty() || name.starts_with('_') {
            continue;
        }
        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(content, match_0.start());
        let body_slice = safe_slice(content, match_0.end(), 3000);

        let node_id = format!("{}::{}", rel_path, name);
        let layer = detect_layer(rel_path, name);
        let stereo = detect_stereotype(name, false, &layer, &[]);

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        for c_cap in re_call_ident().captures_iter(body_slice) {
            let callee = c_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !callee.is_empty() && callee != name && !is_py_keyword(callee) {
                calls.push(RawCall {
                    caller: node_id.clone(),
                    callee: callee.to_string(),
                    relation: "calls".to_string(),
                    order_label: format!("Appel {}", callee),
                });
            }
        }

        for r_cap in re_py_route_str().captures_iter(body_slice) {
            if let Some(ep) = r_cap.get(1) {
                if galaxy == "client" {
                    routes_called.push((ep.as_str().to_string(), node_id.clone()));
                } else {
                    routes_declared.push((ep.as_str().to_string(), node_id.clone()));
                }
            }
        }

        let mut fn_bubble = SymbolBubble::new(
            node_id,
            name.to_string(),
            rel_path.to_string(),
            line_no,
            SymbolKind::Function,
            galaxy.clone(),
            layer,
        );
        fn_bubble.folder = folder.clone();
        fn_bubble.stereotype = stereo;
        fn_bubble.complexity = compute_complexity(body_slice);
        let py_fn_doc = extract_python_docstring(body_slice);
        let fn_doc = if !py_fn_doc.is_empty() {
            py_fn_doc
        } else {
            extract_leading_doc(content, match_0.start(), false)
        };
        fn_bubble.docstring = fn_doc.clone();
        fn_bubble.metadata.methods.push(AstMethod {
            name: name.to_string(),
            args: None,
            returns: None,
            is_async: Some(match_0.as_str().contains("async ")),
            calls: Vec::new(),
            docstring: if fn_doc.is_empty() { None } else { Some(fn_doc) },
        });
        nodes.push(fn_bubble);
    }

    FileAnalysis {
        rel_path: rel_path.to_string(),
        folder,
        raw_content: content.to_string(),
        nodes,
        calls,
        routes_declared,
        routes_called,
        ws_handlers,
        ws_emits,
    }
}

fn analyze_rust(rel_path: &str, content: &str) -> FileAnalysis {
    let mut nodes = Vec::new();
    let mut calls = Vec::new();
    let mut routes_declared = Vec::new();
    let mut routes_called = Vec::new();
    let mut ws_handlers = Vec::new();
    let mut ws_emits = Vec::new();

    let galaxy = detect_galaxy(rel_path);
    let file_node_id = format!("file::{}", rel_path);
    let file_layer = detect_layer(rel_path, "");
    let file_name = Path::new(rel_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());
    let folder = folder_from_rel_path(rel_path);

    let mut file_bubble = SymbolBubble::new(
        file_node_id.clone(),
        file_name,
        rel_path.to_string(),
        1,
        SymbolKind::File,
        galaxy.clone(),
        file_layer,
    );
    file_bubble.folder = folder.clone();
    file_bubble.docstring = format!("Module Rust {}", rel_path);
    file_bubble.lines_count = content.lines().count();
    file_bubble.complexity = compute_complexity(content);
    file_bubble.metadata.lines = file_bubble.lines_count;
    file_bubble.metadata.file_type = "rust".to_string();
    nodes.push(file_bubble);

    // Strip inline #[cfg(test)] unit test blocks from architectural AST symbols
    let prod_content = content
        .find("\n#[cfg(test)]")
        .or_else(|| content.find("\r\n#[cfg(test)]"))
        .map(|idx| &content[..idx])
        .unwrap_or(content);

    // 1. Rust `mod` & `use` declarations (cross-file & cross-crate links)
    for cap in re_rs_mod().captures_iter(prod_content) {
        if let Some(m) = cap.get(1) {
            let mod_name = m.as_str();
            calls.push(RawCall {
                caller: file_node_id.clone(),
                callee: mod_name.to_string(),
                relation: "imports".to_string(),
                order_label: format!("Mod {}", mod_name),
            });
        }
    }

    for cap in re_rs_use().captures_iter(prod_content) {
        if let Some(m) = cap.get(1) {
            let expr = m.as_str().trim();
            // Extract braced symbols if present: foo::bar::{A, B}
            if let Some(brace_start) = expr.find('{') {
                let prefix = expr[..brace_start].trim_end_matches(':');
                for seg in prefix.split("::") {
                    let s = seg.trim();
                    if !s.is_empty() && s != "crate" && s != "self" && s != "super" && s != "std" {
                        calls.push(RawCall {
                            caller: file_node_id.clone(),
                            callee: s.to_string(),
                            relation: "imports".to_string(),
                            order_label: format!("Use {}", s),
                        });
                    }
                }
                if let Some(brace_end) = expr.rfind('}') {
                    let inner = &expr[brace_start + 1..brace_end];
                    for item in inner.split(',') {
                        let clean = item.split(" as ").next().unwrap_or("").trim();
                        if !clean.is_empty() && clean != "*" && clean != "self" {
                            calls.push(RawCall {
                                caller: file_node_id.clone(),
                                callee: clean.to_string(),
                                relation: "imports".to_string(),
                                order_label: format!("Use {}", clean),
                            });
                        }
                    }
                }
            } else {
                for seg in expr.split("::") {
                    let clean = seg.split(" as ").next().unwrap_or("").trim();
                    if !clean.is_empty()
                        && clean != "*"
                        && clean != "crate"
                        && clean != "self"
                        && clean != "super"
                        && clean != "std"
                    {
                        calls.push(RawCall {
                            caller: file_node_id.clone(),
                            callee: clean.to_string(),
                            relation: "imports".to_string(),
                            order_label: format!("Use {}", clean),
                        });
                    }
                }
            }
        }
    }

    // 2. Rust Structs, Enums, Traits
    for cap in re_rs_struct().captures_iter(prod_content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(prod_content, match_0.start());
        let node_id = format!("{}::{}", rel_path, name);
        let layer = detect_layer(rel_path, name);
        let stereo = detect_stereotype(name, true, &layer, &[]);

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        // Extract struct fields for UML card
        let mut attributes = Vec::new();
        let after_slice = safe_slice(prod_content, match_0.end(), 1500);
        if let Some(open_brace) = after_slice.find('{') {
            if open_brace < 120 {
                let body_end = after_slice[open_brace..]
                    .find('}')
                    .unwrap_or(after_slice.len() - open_brace);
                let struct_body = &after_slice[open_brace..open_brace + body_end];
                for f_cap in re_rs_field().captures_iter(struct_body) {
                    let f_name = f_cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let f_type = f_cap.get(2).map(|m| m.as_str().trim().to_string());
                    if !f_name.is_empty() {
                        attributes.push(AstAttribute {
                            name: f_name.to_string(),
                            r#type: f_type,
                        });
                    }
                }
            }
        }

        let mut class_bubble = SymbolBubble::new(
            node_id,
            name.to_string(),
            rel_path.to_string(),
            line_no,
            SymbolKind::ClassOrStruct,
            galaxy.clone(),
            layer,
        );
        class_bubble.folder = folder.clone();
        class_bubble.stereotype = stereo;
        class_bubble.complexity = (1 + attributes.len() as u32 / 2).max(1);
        class_bubble.metadata.attributes = attributes;
        class_bubble.docstring = extract_leading_doc(prod_content, match_0.start(), true);
        nodes.push(class_bubble);
    }

    // Trait implementations -> inherits edge
    for cap in re_rs_impl_trait().captures_iter(prod_content) {
        let trait_name = cap
            .get(1)
            .map(|m| m.as_str().split("::").last().unwrap_or(m.as_str()))
            .unwrap_or("");
        let struct_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        if !trait_name.is_empty() && !struct_name.is_empty() {
            calls.push(RawCall {
                caller: format!("{}::{}", rel_path, struct_name),
                callee: trait_name.to_string(),
                relation: "inherits".to_string(),
                order_label: format!("Implémente {}", trait_name),
            });
        }
    }

    // 3. Rust Functions (`fn ...`)
    for cap in re_rs_fn().captures_iter(prod_content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if name.is_empty() || name.starts_with("test_") || (name == "main" && rel_path.contains("build.rs")) {
            continue;
        }
        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(prod_content, match_0.start());
        let args_str = cap.get(2).map(|m| m.as_str().trim().to_string());
        let ret_str = cap.get(3).map(|m| m.as_str().trim().to_string());
        let body_slice = extract_fn_body_slice(prod_content, match_0.end(), 3500);

        let node_id = format!("{}::{}", rel_path, name);
        let layer = detect_layer(rel_path, name);
        let stereo = detect_stereotype(name, false, &layer, &[]);

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        let mut seen_local_calls: HashSet<String> = HashSet::new();

        for q_cap in re_rs_qual_call().captures_iter(body_slice) {
            let prefix = q_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let method = q_cap.get(2).map(|m| m.as_str()).unwrap_or("");
            if !prefix.is_empty() && !is_rust_keyword(prefix) && prefix != "Self" && prefix != "std" {
                if seen_local_calls.insert(prefix.to_string()) {
                    calls.push(RawCall {
                        caller: node_id.clone(),
                        callee: prefix.to_string(),
                        relation: "calls".to_string(),
                        order_label: format!("Appel {}::{}", prefix, method),
                    });
                }
            }
            if !method.is_empty() && !is_rust_keyword(method) {
                if seen_local_calls.insert(method.to_string()) {
                    calls.push(RawCall {
                        caller: node_id.clone(),
                        callee: method.to_string(),
                        relation: "calls".to_string(),
                        order_label: format!("Appel {}", method),
                    });
                }
            }
        }

        for c_cap in re_call_ident().captures_iter(body_slice) {
            let callee = c_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !callee.is_empty() && callee != name && !is_rust_keyword(callee) {
                if seen_local_calls.insert(callee.to_string()) {
                    calls.push(RawCall {
                        caller: node_id.clone(),
                        callee: callee.to_string(),
                        relation: "calls".to_string(),
                        order_label: format!("Appel {}", callee),
                    });
                }
            }
        }

        for r_cap in re_rs_route().captures_iter(body_slice) {
            if let Some(m) = r_cap.get(1).or_else(|| r_cap.get(2)) {
                let ep = m.as_str().to_string();
                if galaxy == "client" {
                    routes_called.push((ep.clone(), node_id.clone()));
                } else {
                    routes_declared.push((ep.clone(), node_id.clone()));
                }
            }
        }

        if galaxy == "server" && (name.starts_with("handle_") || rel_path.contains("api")) {
            routes_declared.push((name.to_string(), node_id.clone()));
        }

        for ws_cap in re_rs_ws().captures_iter(body_slice) {
            if let Some(m) = ws_cap.get(1) {
                let pkt = m.as_str().to_string();
                if galaxy == "client" {
                    ws_emits.push((pkt, node_id.clone()));
                } else {
                    ws_handlers.push((pkt, node_id.clone()));
                }
            }
        }

        let mut fn_bubble = SymbolBubble::new(
            node_id,
            name.to_string(),
            rel_path.to_string(),
            line_no,
            SymbolKind::Function,
            galaxy.clone(),
            layer,
        );
        fn_bubble.folder = folder.clone();
        fn_bubble.stereotype = stereo;
        fn_bubble.complexity = compute_complexity(body_slice);
        let fn_doc = extract_leading_doc(prod_content, match_0.start(), true);
        fn_bubble.docstring = fn_doc.clone();
        fn_bubble.metadata.methods.push(AstMethod {
            name: name.to_string(),
            args: args_str,
            returns: ret_str,
            is_async: Some(match_0.as_str().contains("async ")),
            calls: Vec::new(),
            docstring: if fn_doc.is_empty() { None } else { Some(fn_doc) },
        });
        nodes.push(fn_bubble);
    }

    FileAnalysis {
        rel_path: rel_path.to_string(),
        folder,
        raw_content: content.to_string(),
        nodes,
        calls,
        routes_declared,
        routes_called,
        ws_handlers,
        ws_emits,
    }
}

fn analyze_cpp(rel_path: &str, content: &str) -> FileAnalysis {
    let mut nodes = Vec::new();
    let mut calls = Vec::new();

    let galaxy = detect_galaxy(rel_path);
    let file_node_id = format!("file::{}", rel_path);
    let file_layer = detect_layer(rel_path, "");
    let file_name = Path::new(rel_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());
    let folder = folder_from_rel_path(rel_path);

    let lines_count = content.lines().count();
    let is_header = rel_path.ends_with(".h")
        || rel_path.ends_with(".hpp")
        || rel_path.ends_with(".hh")
        || rel_path.ends_with(".hxx");

    let mut file_bubble = SymbolBubble::new(
        file_node_id.clone(),
        file_name.clone(),
        rel_path.to_string(),
        1,
        SymbolKind::File,
        galaxy.clone(),
        file_layer,
    );
    file_bubble.folder = folder.clone();
    file_bubble.stereotype = if is_header {
        "«Header»".to_string()
    } else {
        "«File»".to_string()
    };
    file_bubble.docstring = format!("Fichier source C/C++ {}", rel_path);
    file_bubble.lines_count = lines_count;
    file_bubble.complexity = compute_complexity(content);
    file_bubble.metadata.lines = lines_count;
    file_bubble.metadata.file_type = "cpp".to_string();
    nodes.push(file_bubble);

    let cur_dir = Path::new(rel_path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();

    // 1. Includes
    for cap in re_cpp_include().captures_iter(content) {
        let is_quoted = cap.get(1).map(|m| m.as_str() == "\"").unwrap_or(false);
        if let Some(target_match) = cap.get(2) {
            let target_path = target_match.as_str();
            let norm_target = if is_quoted && !target_path.starts_with('/') {
                if cur_dir.is_empty() {
                    target_path.to_string()
                } else {
                    normalize_path(&format!("{}/{}", cur_dir, target_path))
                }
            } else {
                target_path.to_string()
            };

            calls.push(RawCall {
                caller: file_node_id.clone(),
                callee: norm_target,
                relation: "imports".to_string(),
                order_label: format!("Include {}", target_path),
            });

            let stem = Path::new(target_path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if !stem.is_empty() && stem != file_name {
                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: stem.clone(),
                    relation: "imports".to_string(),
                    order_label: format!("Include {}", stem),
                });
            }
        }
    }

    // 2. Classes & Structs
    for cap in re_cpp_class().captures_iter(content) {
        let class_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if class_name.is_empty() || is_cpp_keyword(class_name) {
            continue;
        }
        let match_0 = cap.get(0).unwrap();
        let line_no = line_number_at(content, match_0.start());

        let bases_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let bases: Vec<String> = bases_str
            .split(',')
            .map(|s| {
                s.replace("public", "")
                    .replace("protected", "")
                    .replace("private", "")
                    .replace("virtual", "")
                    .trim()
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        let node_id = format!("{}::{}", rel_path, class_name);
        let layer = detect_layer(rel_path, class_name);
        let stereo = detect_stereotype(class_name, true, &layer, &bases);

        calls.push(RawCall {
            caller: file_node_id.clone(),
            callee: node_id.clone(),
            relation: "contains".to_string(),
            order_label: "Contient".to_string(),
        });

        for base in &bases {
            calls.push(RawCall {
                caller: node_id.clone(),
                callee: base.clone(),
                relation: "inherits".to_string(),
                order_label: format!("Hérite de {}", base),
            });
        }

        let mut attributes = Vec::new();
        let mut methods = Vec::new();

        let class_body_slice = safe_slice(content, match_0.end(), 8000);
        let class_body = if let Some(close_idx) = class_body_slice.find("};") {
            safe_slice(class_body_slice, 0, close_idx)
        } else {
            safe_slice(class_body_slice, 0, 2000)
        };

        for m_cap in re_cpp_member_method().captures_iter(class_body) {
            let m_name = m_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !m_name.is_empty() && !is_cpp_keyword(m_name) {
                let args = m_cap.get(2).map(|m| m.as_str().to_string());
                methods.push(AstMethod {
                    name: m_name.to_string(),
                    args,
                    returns: None,
                    is_async: None,
                    calls: Vec::new(),
                    docstring: None,
                });
            }
        }

        for f_cap in re_cpp_member_field().captures_iter(class_body) {
            let f_name = f_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !f_name.is_empty() && !is_cpp_keyword(f_name) {
                attributes.push(AstAttribute {
                    name: f_name.to_string(),
                    r#type: None,
                });
            }
        }

        let mut class_bubble = SymbolBubble::new(
            node_id,
            class_name.to_string(),
            rel_path.to_string(),
            line_no,
            SymbolKind::ClassOrStruct,
            galaxy.clone(),
            layer,
        );
        class_bubble.folder = folder.clone();
        class_bubble.stereotype = stereo;
        class_bubble.complexity = compute_complexity(class_body);
        class_bubble.docstring = format!("Classe/Structure C++ {}", class_name);
        class_bubble.metadata.attributes = attributes;
        class_bubble.metadata.methods = methods;
        nodes.push(class_bubble);
    }

    // 3. Functions & Methods
    let mut seen_funcs = HashSet::new();

    for cap in re_cpp_func().captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let full_func_name = m.as_str();
            let short_name = full_func_name.split("::").last().unwrap_or(full_func_name);
            if is_cpp_keyword(short_name) || is_cpp_keyword(full_func_name) {
                continue;
            }
            if !seen_funcs.insert(full_func_name.to_string()) {
                continue;
            }
            let match_0 = cap.get(0).unwrap();
            let line_no = line_number_at(content, match_0.start());
            let node_id = format!("{}::{}", rel_path, full_func_name);
            let layer = detect_layer(rel_path, full_func_name);
            let stereo = if full_func_name.contains("::") {
                "«Method»".to_string()
            } else {
                detect_stereotype(short_name, false, &layer, &[])
            };

            calls.push(RawCall {
                caller: file_node_id.clone(),
                callee: node_id.clone(),
                relation: "contains".to_string(),
                order_label: "Contient".to_string(),
            });

            if let Some((class_name, _)) = full_func_name.split_once("::") {
                calls.push(RawCall {
                    caller: format!("{}::{}", rel_path, class_name),
                    callee: node_id.clone(),
                    relation: "contains".to_string(),
                    order_label: "Méthode".to_string(),
                });
            }

            let body_slice = safe_slice(content, match_0.end(), 4000);
            for c_cap in re_cpp_call().captures_iter(body_slice) {
                let callee = c_cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let callee_short = callee.split("::").last().unwrap_or(callee);
                if !callee.is_empty()
                    && callee != full_func_name
                    && callee_short != short_name
                    && !is_cpp_keyword(callee_short)
                {
                    calls.push(RawCall {
                        caller: node_id.clone(),
                        callee: callee_short.to_string(),
                        relation: "calls".to_string(),
                        order_label: format!("Appel {}", callee_short),
                    });
                }
            }

            let mut fn_bubble = SymbolBubble::new(
                node_id,
                full_func_name.to_string(),
                rel_path.to_string(),
                line_no,
                SymbolKind::Function,
                galaxy.clone(),
                layer,
            );
            fn_bubble.folder = folder.clone();
            fn_bubble.stereotype = stereo;
            fn_bubble.complexity = compute_complexity(body_slice);
            fn_bubble.docstring = format!("Fonction/Méthode C++ {}", full_func_name);
            nodes.push(fn_bubble);
        }
    }

    FileAnalysis {
        rel_path: rel_path.to_string(),
        folder,
        raw_content: content.to_string(),
        nodes,
        calls,
        routes_declared: Vec::new(),
        routes_called: Vec::new(),
        ws_handlers: Vec::new(),
        ws_emits: Vec::new(),
    }
}

fn analyze_markdown(rel_path: &str, content: &str) -> FileAnalysis {
    let mut nodes = Vec::new();
    let mut calls = Vec::new();

    let galaxy = detect_galaxy(rel_path);
    let file_node_id = format!("file::{}", rel_path);
    let folder = folder_from_rel_path(rel_path);

    let mut doc_title = String::new();
    let mut sections = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            if doc_title.is_empty() {
                doc_title = trimmed.trim_start_matches("# ").trim().to_string();
            }
            sections.push(AstMethod {
                name: trimmed.to_string(),
                args: None,
                returns: None,
                is_async: None,
                calls: Vec::new(),
                docstring: None,
            });
        } else if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            sections.push(AstMethod {
                name: trimmed.to_string(),
                args: None,
                returns: None,
                is_async: None,
                calls: Vec::new(),
                docstring: None,
            });
        }
    }

    if doc_title.is_empty() {
        doc_title = Path::new(rel_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| rel_path.to_string());
    }

    let mut doc_bubble = SymbolBubble::new(
        file_node_id.clone(),
        doc_title,
        rel_path.to_string(),
        1,
        SymbolKind::File,
        galaxy,
        "docs".to_string(),
    );
    doc_bubble.folder = folder.clone();
    doc_bubble.stereotype = "«Document»".to_string();
    doc_bubble.symbol_type = "document".to_string();
    doc_bubble.docstring = format!("Documentation Markdown : {}", rel_path);
    doc_bubble.lines_count = content.lines().count();
    doc_bubble.complexity = (1 + sections.len() as u32 / 2).max(1);
    doc_bubble.metadata.lines = doc_bubble.lines_count;
    doc_bubble.metadata.file_type = "markdown".to_string();
    doc_bubble.metadata.methods = sections;
    // Keep raw_content for READMEs under 32 KB so doc impact zones & preview work immediately
    if content.len() <= 32_768 {
        doc_bubble.metadata.raw_content = Some(content.to_string());
    }
    nodes.push(doc_bubble);

    let cur_dir = Path::new(rel_path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();

    // 1. Markdown links: [Text](path/to/file)
    for cap in re_md_link().captures_iter(content) {
        if let Some(target) = cap.get(2) {
            let t_str = target.as_str().trim();
            if !t_str.starts_with("http://")
                && !t_str.starts_with("https://")
                && !t_str.starts_with('#')
            {
                let clean_target = t_str.split('#').next().unwrap_or(t_str);
                let resolved = if clean_target.starts_with('/') {
                    clean_target.trim_start_matches('/').to_string()
                } else if !cur_dir.is_empty() {
                    format!("{}/{}", cur_dir, clean_target)
                } else {
                    clean_target.to_string()
                };
                let norm = normalize_path(&resolved);
                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: format!("file::{}", norm),
                    relation: "documents".to_string(),
                    order_label: "Documente".to_string(),
                });
            }
        }
    }

    // 2. Obsidian Wikilinks: [[Target]]
    for cap in re_md_wiki().captures_iter(content) {
        if let Some(target) = cap.get(1) {
            let t_name = target.as_str().trim();
            if !t_name.is_empty() {
                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: t_name.to_string(),
                    relation: "documents".to_string(),
                    order_label: "Documente".to_string(),
                });
            }
        }
    }

    // 3. Inline code backticks: `SymbolName`
    let ignore_words = [
        "true",
        "false",
        "null",
        "undefined",
        "const",
        "let",
        "var",
        "function",
        "string",
        "number",
        "boolean",
        "async",
        "await",
        "import",
        "export",
        "npm",
        "pnpm",
        "yarn",
        "bash",
        "json",
        "rust",
        "python",
        "test",
        "cargo",
        "self",
    ];
    for cap in re_md_code().captures_iter(content) {
        if let Some(word) = cap.get(1) {
            let w = word.as_str();
            if !ignore_words.contains(&w.to_lowercase().as_str()) {
                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: w.to_string(),
                    relation: "documents".to_string(),
                    order_label: "Documente".to_string(),
                });
            }
        }
    }

    FileAnalysis {
        rel_path: rel_path.to_string(),
        folder,
        raw_content: content.to_string(),
        nodes,
        calls,
        routes_declared: Vec::new(),
        routes_called: Vec::new(),
        ws_handlers: Vec::new(),
        ws_emits: Vec::new(),
    }
}

fn analyze_solidity(rel_path: &str, content: &str) -> FileAnalysis {
    let mut nodes = Vec::new();
    let mut calls = Vec::new();
    let routes_declared = Vec::new();
    let routes_called = Vec::new();

    let galaxy = detect_galaxy(rel_path);
    let file_node_id = format!("file::{}", rel_path);
    let file_layer = detect_layer(rel_path, "");
    let file_name = Path::new(rel_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());
    let folder = folder_from_rel_path(rel_path);

    let lines_count = content.lines().count();

    let mut file_bubble = SymbolBubble::new(
        file_node_id.clone(),
        file_name.clone(),
        rel_path.to_string(),
        1,
        SymbolKind::File,
        galaxy.clone(),
        file_layer.clone(),
    );
    file_bubble.folder = folder.clone();
    file_bubble.stereotype = "«Solidity»".to_string();
    file_bubble.docstring = format!("Contrat / Module Solidity {}", rel_path);
    file_bubble.lines_count = lines_count;
    file_bubble.complexity = compute_complexity(content);
    file_bubble.metadata.lines = lines_count;
    file_bubble.metadata.file_type = "solidity".to_string();
    nodes.push(file_bubble);

    let cur_dir = Path::new(rel_path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();

    // 1. Imports
    for cap in re_sol_import().captures_iter(content) {
        if let Some(target_match) = cap.get(1) {
            let target_path = target_match.as_str();
            let norm_target = if !target_path.starts_with('/') && !target_path.starts_with('@') {
                if cur_dir.is_empty() {
                    target_path.to_string()
                } else {
                    normalize_path(&format!("{}/{}", cur_dir, target_path))
                }
            } else {
                target_path.to_string()
            };

            calls.push(RawCall {
                caller: file_node_id.clone(),
                callee: norm_target,
                relation: "imports".to_string(),
                order_label: format!("Import {}", target_path),
            });

            let stem = Path::new(target_path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if !stem.is_empty() && stem != file_name {
                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: stem.clone(),
                    relation: "imports".to_string(),
                    order_label: format!("Import {}", stem),
                });
            }
        }
    }

    // 2. Contracts / Interfaces / Libraries
    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        if let Some(cap) = re_sol_contract().captures(line) {
            let is_abstract = cap.get(1).is_some();
            let kind_str = cap.get(2).map(|m| m.as_str()).unwrap_or("contract");
            let contract_name = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let inheritance = cap.get(4).map(|m| m.as_str()).unwrap_or("");

            if !contract_name.is_empty() {
                let c_node_id = format!("{}::{}", file_node_id, contract_name);
                let mut c_bubble = SymbolBubble::new(
                    c_node_id.clone(),
                    contract_name.to_string(),
                    rel_path.to_string(),
                    line_num,
                    SymbolKind::Contract,
                    galaxy.clone(),
                    file_layer.clone(),
                );
                c_bubble.folder = folder.clone();
                c_bubble.stereotype = if is_abstract {
                    "«AbstractContract»".to_string()
                } else {
                    match kind_str {
                        "interface" => "«Interface»".to_string(),
                        "library" => "«Library»".to_string(),
                        _ => "«Contract»".to_string(),
                    }
                };
                c_bubble.docstring = format!("{} Solidity {}", kind_str, contract_name);
                nodes.push(c_bubble);

                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: c_node_id.clone(),
                    relation: "contains".to_string(),
                    order_label: "Définit".to_string(),
                });

                if !inheritance.is_empty() {
                    for parent in inheritance.split(',') {
                        let p_name = parent.trim();
                        if !p_name.is_empty() {
                            calls.push(RawCall {
                                caller: c_node_id.clone(),
                                callee: p_name.to_string(),
                                relation: "inherits".to_string(),
                                order_label: format!("Hérite de {}", p_name),
                            });
                        }
                    }
                }
            }
        } else if let Some(cap) = re_sol_func().captures(line) {
            let func_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let params = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            if !func_name.is_empty() {
                let fn_node_id = format!("{}::fn::{}", file_node_id, func_name);
                let mut fn_bubble = SymbolBubble::new(
                    fn_node_id.clone(),
                    func_name.to_string(),
                    rel_path.to_string(),
                    line_num,
                    SymbolKind::Function,
                    galaxy.clone(),
                    file_layer.clone(),
                );
                fn_bubble.folder = folder.clone();
                fn_bubble.stereotype = "«Function»".to_string();
                fn_bubble.docstring = format!("function {}({})", func_name, params);
                nodes.push(fn_bubble);

                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: fn_node_id.clone(),
                    relation: "contains".to_string(),
                    order_label: "Fonction".to_string(),
                });
            }
        } else if let Some(cap) = re_sol_modifier().captures(line) {
            let mod_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !mod_name.is_empty() {
                let mod_node_id = format!("{}::mod::{}", file_node_id, mod_name);
                let mut mod_bubble = SymbolBubble::new(
                    mod_node_id.clone(),
                    mod_name.to_string(),
                    rel_path.to_string(),
                    line_num,
                    SymbolKind::Function,
                    galaxy.clone(),
                    file_layer.clone(),
                );
                mod_bubble.folder = folder.clone();
                mod_bubble.stereotype = "«Modifier»".to_string();
                mod_bubble.docstring = format!("modifier {}", mod_name);
                nodes.push(mod_bubble);

                calls.push(RawCall {
                    caller: file_node_id.clone(),
                    callee: mod_node_id.clone(),
                    relation: "contains".to_string(),
                    order_label: "Modifier".to_string(),
                });
            }
        }
    }

    // 3. Appels de fonctions internes / inter-contrats
    for (_line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        for cap in re_sol_call().captures_iter(line) {
            if let Some(target) = cap.get(1) {
                let t_name = target.as_str();
                let ignore = ["require", "assert", "revert", "keccak256", "abi", "type", "if", "for", "while", "return", "returns", "emit", "super", "assembly"];
                if !ignore.contains(&t_name) && t_name.len() > 2 {
                    calls.push(RawCall {
                        caller: file_node_id.clone(),
                        callee: t_name.to_string(),
                        relation: "calls".to_string(),
                        order_label: format!("Appel {}", t_name),
                    });
                }
            }
        }
    }

    FileAnalysis {
        rel_path: rel_path.to_string(),
        folder,
        raw_content: content.to_string(),
        nodes,
        calls,
        routes_declared,
        routes_called,
        ws_handlers: Vec::new(),
        ws_emits: Vec::new(),
    }
}

pub fn resolve_graph_edges(
    nodes: &mut Vec<SymbolBubble>,
    raw_calls: &[RawCall],
) -> Vec<AffiliationEdge> {
    let mut node_by_id: HashMap<String, String> = HashMap::with_capacity(nodes.len());
    let mut node_idx_by_id: HashMap<String, usize> = HashMap::with_capacity(nodes.len());
    let mut node_by_filepath: HashMap<String, String> = HashMap::new();
    let mut node_by_name: HashMap<String, String> = HashMap::with_capacity(nodes.len());
    let mut node_galaxy: HashMap<String, String> = HashMap::with_capacity(nodes.len());
    let mut db_node_ids: HashSet<String> = HashSet::new();

    for (idx, n) in nodes.iter().enumerate() {
        node_by_id.insert(n.id.clone(), n.id.clone());
        node_idx_by_id.insert(n.id.clone(), idx);
        node_galaxy.insert(n.id.clone(), n.galaxy.clone());

        if n.layer == "database"
            || n.folder == "data-nodes"
            || n.file_path.contains("/db/")
            || n.file_path.contains("/database/")
        {
            db_node_ids.insert(n.id.clone());
        }

        if n.is_file {
            let fp = n.file_path.replace('\\', "/");
            node_by_filepath.insert(fp.clone(), n.id.clone());

            let p = Path::new(&fp);
            if let Some(stem) = p.file_stem() {
                let stem_str = stem.to_string_lossy().to_string();
                node_by_name
                    .entry(stem_str.clone())
                    .or_insert_with(|| n.id.clone());
                if let Some(parent) = p.parent() {
                    let no_ext = parent.join(stem).to_string_lossy().replace('\\', "/");
                    node_by_filepath.insert(no_ext.clone(), n.id.clone());
                    node_by_filepath.insert(format!("{}/index", no_ext), n.id.clone());
                }
            }
        }

        let existing = node_by_name.get(&n.name);
        if existing.is_none()
            || n.is_class
            || n.symbol_type == "component"
            || n.symbol_type == "form"
        {
            node_by_name.insert(n.name.clone(), n.id.clone());
        }
    }

    let mut resolved_edges = Vec::new();
    let mut seen_edges: HashSet<(String, String, String)> = HashSet::new();

    for call in raw_calls {
        let src = &call.caller;
        let callee = &call.callee;

        let mut tgt = node_by_id
            .get(callee)
            .or_else(|| node_by_filepath.get(callee))
            .or_else(|| node_by_name.get(callee))
            .cloned();

        // Fuzzy match for API endpoints
        if tgt.is_none()
            && (call.relation == "network"
                || callee.starts_with("/api")
                || callee.starts_with("api/"))
        {
            let clean_ep = callee.trim_start_matches('/');
            for (fp, id) in &node_by_filepath {
                if fp.contains(clean_ep) {
                    tgt = Some(id.clone());
                    break;
                }
            }
        }

        // Materialize dynamic API endpoint node for unresolved network calls
        if tgt.is_none()
            && call.relation == "network"
            && callee.len() > 1
            && !callee.contains('{')
        {
            let ep_id = format!("api::{}", callee);
            let ep_name = format!("🌐 {}", callee);
            let mut ep_bubble = SymbolBubble::new(
                ep_id.clone(),
                ep_name,
                callee.clone(),
                1,
                SymbolKind::FormOrEndpoint,
                "server".to_string(),
                "action".to_string(),
            );
            ep_bubble.folder = "api".to_string();
            ep_bubble.stereotype = "«API Endpoint»".to_string();
            ep_bubble.symbol_type = "endpoint".to_string();
            ep_bubble.docstring = format!("Point d'accès réseau : {}", callee);
            let new_idx = nodes.len();
            nodes.push(ep_bubble);
            node_by_id.insert(callee.clone(), ep_id.clone());
            node_idx_by_id.insert(ep_id.clone(), new_idx);
            tgt = Some(ep_id);
        }

        if let Some(target_id) = tgt {
            if target_id != *src {
                let key = (src.clone(), target_id.clone(), call.relation.clone());
                if seen_edges.insert(key) {
                    let (edge_type, mut filters) = match call.relation.as_str() {
                        "contains" => (EdgeType::Contains, FilterAffiliation::PHYSICAL),
                        "imports" => (EdgeType::Import, FilterAffiliation::PHYSICAL),
                        "calls" => (
                            EdgeType::Call,
                            FilterAffiliation::PHYSICAL | FilterAffiliation::SEMANTIC,
                        ),
                        "renders" => (
                            EdgeType::Render,
                            FilterAffiliation::PHYSICAL | FilterAffiliation::SEMANTIC,
                        ),
                        "inherits" => (
                            EdgeType::Inherit,
                            FilterAffiliation::PHYSICAL | FilterAffiliation::SEMANTIC,
                        ),
                        "network" => (
                            EdgeType::Network,
                            FilterAffiliation::NETWORK_RPC | FilterAffiliation::PHYSICAL,
                        ),
                        "submits" => (
                            EdgeType::NetworkRPC,
                            FilterAffiliation::NETWORK_RPC | FilterAffiliation::PHYSICAL,
                        ),
                        "documents" => (EdgeType::Documents, FilterAffiliation::SEMANTIC),
                        "queries" => (
                            EdgeType::DatabaseQuery,
                            FilterAffiliation::DATABASE_QUERY | FilterAffiliation::PHYSICAL,
                        ),
                        _ => {
                            if db_node_ids.contains(&target_id) {
                                (
                                    EdgeType::DatabaseQuery,
                                    FilterAffiliation::DATABASE_QUERY | FilterAffiliation::PHYSICAL,
                                )
                            } else {
                                (EdgeType::Call, FilterAffiliation::PHYSICAL)
                            }
                        }
                    };

                    // Detect cross-galaxy illegal import (e.g. client importing server directly)
                    let mut violation_reason = None;
                    if call.relation == "imports" {
                        let s_gal = node_galaxy.get(src).map(|s| s.as_str()).unwrap_or("");
                        let t_gal = node_galaxy
                            .get(&target_id)
                            .map(|s| s.as_str())
                            .unwrap_or("");
                        if (s_gal == "client" && (t_gal == "server" || t_gal == "data-nodes"))
                            || (s_gal == "contracts" && (t_gal == "client" || t_gal == "server"))
                        {
                            filters |= FilterAffiliation::SECURITY_VIOLATES;
                            let reason = format!(
                                "[CROSS_GALAXY_VIOLATION] Import inter-galaxie illégal: '{}' ({}) -> '{}' ({})",
                                src, s_gal, target_id, t_gal
                            );
                            violation_reason = Some(reason.clone());
                            if let Some(&src_idx) = node_idx_by_id.get(src) {
                                let src_node = &mut nodes[src_idx];
                                src_node.has_security_violation = true;
                                src_node.metadata.violations.push(reason);
                            }
                        }
                    }

                    let mut edge =
                        AffiliationEdge::new(src.clone(), target_id, edge_type, filters);
                    edge.violation_details = violation_reason;
                    edge.order_label = if call.order_label.is_empty() {
                        format!("{}.", resolved_edges.len() + 1)
                    } else {
                        call.order_label.clone()
                    };
                    resolved_edges.push(edge);
                }
            }
        }
    }

    // Compute in_degree, out_degree, and DAG stage_idx
    let mut in_degrees: HashMap<String, usize> = HashMap::new();
    let mut out_degrees: HashMap<String, usize> = HashMap::new();
    for e in &resolved_edges {
        *in_degrees.entry(e.target.clone()).or_insert(0) += 1;
        *out_degrees.entry(e.source.clone()).or_insert(0) += 1;
    }

    for n in nodes.iter_mut() {
        n.in_degree = in_degrees.get(&n.id).copied().unwrap_or(0);
        n.out_degree = out_degrees.get(&n.id).copied().unwrap_or(0);

        if n.in_degree == 0 && n.out_degree > 0 {
            n.stage_idx = 0; // Entrypoint
        } else if n.out_degree > 0 && n.in_degree > 0 {
            n.stage_idx = 1; // Coordinator
        } else if n.layer == "database" || n.name.to_lowercase().contains("model") {
            n.stage_idx = 3; // Persistence
        } else {
            n.stage_idx = 2; // Processing
        }
    }

    resolved_edges
}

fn line_number_at(content: &str, byte_offset: usize) -> usize {
    let end = byte_offset.min(content.len());
    content[..end].bytes().filter(|&b| b == b'\n').count() + 1
}

fn normalize_path(path_str: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for seg in path_str.split('/') {
        if seg == "." || seg.is_empty() {
            continue;
        } else if seg == ".." {
            parts.pop();
        } else {
            parts.push(seg);
        }
    }
    parts.join("/")
}

fn safe_slice(s: &str, start: usize, len: usize) -> &str {
    if start >= s.len() {
        return "";
    }
    let actual_start = if s.is_char_boundary(start) {
        start
    } else {
        (0..start)
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0)
    };
    let target_end = (actual_start + len).min(s.len());
    let actual_end = if s.is_char_boundary(target_end) {
        target_end
    } else {
        (actual_start..target_end)
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(actual_start)
    };
    &s[actual_start..actual_end]
}

fn extract_fn_body_slice(s: &str, start: usize, max_len: usize) -> &str {
    let window = safe_slice(s, start, max_len);
    let mut depth: i32 = 0;
    let mut opened = false;
    for (idx, b) in window.bytes().enumerate() {
        if !opened && b == b';' && idx < 160 {
            return &window[..idx];
        }
        if b == b'{' {
            depth += 1;
            opened = true;
        } else if b == b'}' && opened {
            depth -= 1;
            if depth <= 0 {
                let end = (idx + 1).min(window.len());
                if window.is_char_boundary(end) {
                    return &window[..end];
                }
                break;
            }
        }
    }
    window
}

fn is_js_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "for"
            | "while"
            | "switch"
            | "catch"
            | "return"
            | "require"
            | "import"
            | "function"
            | "const"
            | "let"
            | "var"
            | "throw"
            | "typeof"
            | "instanceof"
            | "new"
            | "case"
            | "default"
            | "super"
    )
}

fn is_py_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "elif"
            | "for"
            | "while"
            | "return"
            | "yield"
            | "print"
            | "len"
            | "range"
            | "str"
            | "int"
            | "float"
            | "list"
            | "dict"
            | "set"
            | "tuple"
            | "super"
            | "isinstance"
    )
}

fn is_rust_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "match"
            | "for"
            | "while"
            | "loop"
            | "return"
            | "Some"
            | "None"
            | "Ok"
            | "Err"
            | "Box"
            | "Vec"
            | "String"
            | "Arc"
            | "Mutex"
            | "RwLock"
            | "HashMap"
            | "HashSet"
            | "BTreeMap"
            | "BTreeSet"
            | "VecDeque"
            | "Path"
            | "PathBuf"
            | "Duration"
            | "Instant"
            | "Option"
            | "Result"
            | "format"
            | "println"
            | "eprintln"
            | "write"
            | "writeln"
            | "vec"
            | "Default"
            | "default"
            | "new"
            | "with_capacity"
            | "from"
            | "into"
            | "clone"
            | "cloned"
            | "copied"
            | "unwrap"
            | "expect"
            | "map"
            | "map_err"
            | "and_then"
            | "or_else"
            | "ok_or"
            | "ok_or_else"
            | "unwrap_or"
            | "unwrap_or_else"
            | "unwrap_or_default"
            | "collect"
            | "iter"
            | "iter_mut"
            | "into_iter"
            | "par_iter"
            | "to_string"
            | "to_owned"
            | "to_lowercase"
            | "to_uppercase"
            | "to_string_lossy"
            | "as_str"
            | "as_ref"
            | "as_mut"
            | "as_bytes"
            | "as_slice"
            | "as_deref"
            | "len"
            | "is_empty"
            | "push"
            | "push_str"
            | "pop"
            | "insert"
            | "remove"
            | "get"
            | "get_mut"
            | "entry"
            | "or_insert"
            | "or_insert_with"
            | "or_default"
            | "contains"
            | "contains_key"
            | "starts_with"
            | "ends_with"
            | "find"
            | "rfind"
            | "split"
            | "trim"
            | "trim_start"
            | "trim_end"
            | "trim_matches"
            | "trim_start_matches"
            | "trim_end_matches"
            | "replace"
            | "lines"
            | "bytes"
            | "chars"
            | "filter"
            | "filter_map"
            | "any"
            | "all"
            | "enumerate"
            | "take"
            | "skip"
            | "rev"
            | "count"
            | "ok"
            | "err"
            | "is_ok"
            | "is_err"
            | "is_some"
            | "is_none"
            | "min"
            | "max"
            | "clamp"
            | "saturating_sub"
            | "saturating_add"
            | "checked_sub"
            | "checked_add"
            | "lock"
            | "read"
            | "try_recv"
            | "join"
            | "strip_prefix"
            | "extension"
            | "file_name"
            | "file_stem"
            | "parent"
            | "exists"
            | "is_dir"
            | "is_file"
    )
}

fn is_cpp_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "default"
            | "return"
            | "break"
            | "continue"
            | "goto"
            | "try"
            | "catch"
            | "throw"
            | "sizeof"
            | "alignof"
            | "decltype"
            | "typeid"
            | "new"
            | "delete"
            | "static_cast"
            | "dynamic_cast"
            | "reinterpret_cast"
            | "const_cast"
            | "struct"
            | "class"
            | "union"
            | "enum"
            | "namespace"
            | "template"
            | "typename"
            | "public"
            | "private"
            | "protected"
            | "virtual"
            | "override"
            | "final"
            | "const"
            | "static"
            | "inline"
            | "constexpr"
            | "explicit"
            | "friend"
            | "auto"
            | "void"
            | "int"
            | "char"
            | "float"
            | "double"
            | "bool"
            | "short"
            | "long"
            | "unsigned"
            | "signed"
            | "true"
            | "false"
            | "nullptr"
            | "this"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_analysis() {
        let php_code = r#"<?php
namespace App\Controller;

use App\Service\PaymentGateway;
use App\Model\User;

class UserController extends BaseController implements UserInterface {
    public function getProfile($userId) {
        $user = $this->findUser($userId);
        return $user;
    }
}
"#;
        let res = analyze_php("server/controllers/UserController.php", php_code);
        assert!(res.nodes.iter().any(|n| n.is_file && n.name == "UserController.php"));
        assert_eq!(res.nodes[0].metadata.file_type, "php");
        let cls = res.nodes.iter().find(|n| n.name == "UserController").expect("UserController class");
        assert!(cls.is_class);
        assert!(res.calls.iter().any(|c| c.relation == "imports" && c.callee == "PaymentGateway"));
        assert!(res.calls.iter().any(|c| c.relation == "inherits" && c.callee == "BaseController"));
        assert!(res.nodes.iter().any(|n| n.name == "getProfile"));
    }
    #[test]
    fn test_rust_and_markdown_analysis() {
        let rs_code = r#"
pub struct BookingService {
    pub db_url: String,
    pub max_retries: u32,
}

pub fn compute_price(nights: u32) -> u64 {
    validate_nights(nights);
    nights as u64 * 100
}

pub fn validate_nights(n: u32) -> bool {
    n > 0
}
"#;
        let res = analyze_rust("server/domain/booking.rs", rs_code);
        assert!(res.nodes.iter().any(|n| n.is_file && n.name == "booking.rs"));
        let st = res
            .nodes
            .iter()
            .find(|n| n.name == "BookingService")
            .expect("BookingService struct");
        assert!(st.is_class);
        assert_eq!(st.metadata.attributes.len(), 2);
        assert!(res
            .calls
            .iter()
            .any(|c| c.relation == "contains" && c.callee.ends_with("BookingService")));
        assert!(res
            .calls
            .iter()
            .any(|c| c.relation == "calls" && c.callee == "validate_nights"));
    }

    #[test]
    fn test_scanner_with_exclusions() {
        let temp_dir = std::env::temp_dir().join(format!("aether_test_excl_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let src_dir = temp_dir.join("src");
        let tests_dir = temp_dir.join("tests");
        let mocks_dir = temp_dir.join("mocks");
        let _ = std::fs::create_dir_all(&src_dir);
        let _ = std::fs::create_dir_all(&tests_dir);
        let _ = std::fs::create_dir_all(&mocks_dir);

        let _ = std::fs::write(src_dir.join("main.rs"), "pub fn main() {}");
        let _ = std::fs::write(src_dir.join("ui.spec.ts"), "export const a = 1;");
        let _ = std::fs::write(tests_dir.join("integration.rs"), "pub fn test_run() {}");
        let _ = std::fs::write(mocks_dir.join("db.rs"), "pub fn mock_db() {}");

        let scanner = ProjectScanner::default();
        let exclusions = vec!["tests".to_string(), "*.spec.*".to_string(), "mocks".to_string()];
        let graph = scanner.scan_project_with_exclusions(&temp_dir, &exclusions);

        let found_files: Vec<String> = graph.nodes.iter().filter(|n| n.is_file).map(|n| n.name.clone()).collect();
        assert!(found_files.contains(&"main.rs".to_string()), "main.rs should be scanned");
        assert!(!found_files.contains(&"ui.spec.ts".to_string()), "ui.spec.ts should be excluded");
        assert!(!found_files.contains(&"integration.rs".to_string()), "integration.rs should be excluded");
        assert!(!found_files.contains(&"db.rs".to_string()), "db.rs should be excluded");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
