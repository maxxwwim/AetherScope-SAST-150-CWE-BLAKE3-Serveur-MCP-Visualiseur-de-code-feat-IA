use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymbolKind {
    Function = 0,
    File = 1,
    ClassOrStruct = 2,
    FormOrEndpoint = 3,
    Contract = 4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutPositions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obsidian: Option<Point2D>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Point2D>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<Point2D>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<Point2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AstMethod {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
    #[serde(default)]
    pub calls: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AstAttribute {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CveAnomalyRecord {
    pub rule_id: String,
    pub cve_id: String,
    pub severity: String,
    pub line_number: usize,
    pub file_path: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted_type: Option<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolMetadata {
    #[serde(default)]
    pub lines: usize,
    #[serde(default)]
    pub file_type: String,
    #[serde(default)]
    pub methods: Vec<AstMethod>,
    #[serde(default)]
    pub attributes: Vec<AstAttribute>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blake3_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock_verified: Option<bool>,
    #[serde(default)]
    pub ears_count: usize,
    #[serde(default)]
    pub violations: Vec<String>,
    #[serde(default)]
    pub cve_anomalies: Vec<CveAnomalyRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolBubble {
    pub id: String,
    pub name: String,
    pub file_path: String,
    #[serde(default)]
    pub folder: String,
    pub line_number: usize,
    pub kind: SymbolKind,
    pub galaxy: String, // "contracts" | "client" | "server" | "data-nodes"
    pub layer: String,  // "ui" | "action" | "domain" | "database" | "utility" | "config" | "docs"
    pub stereotype: String,
    pub symbol_type: String,
    #[serde(default)]
    pub docstring: String,
    pub is_class: bool,
    pub is_file: bool,
    pub side: String,
    pub in_degree: usize,
    pub out_degree: usize,
    pub stage_idx: usize,
    pub complexity: u32,
    pub lines_count: usize,
    pub is_dead_code: bool,
    pub has_security_violation: bool,
    pub color_rgba: [f32; 4],
    pub radius: f32,
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_time: Option<f64>,
    pub layouts: LayoutPositions,
    #[serde(default)]
    pub metadata: SymbolMetadata,
}

impl SymbolBubble {
    pub fn new(
        id: String,
        name: String,
        file_path: String,
        line_number: usize,
        kind: SymbolKind,
        galaxy: String,
        layer: String,
    ) -> Self {
        let (color, radius) = match kind {
            SymbolKind::Function => ([0.22, 0.74, 0.97, 1.0], 5.0),
            SymbolKind::File => ([0.63, 0.52, 0.95, 1.0], 14.0),
            SymbolKind::ClassOrStruct => ([0.13, 0.77, 0.36, 1.0], 10.0),
            SymbolKind::FormOrEndpoint => ([0.96, 0.62, 0.13, 1.0], 9.0),
            SymbolKind::Contract => ([0.94, 0.27, 0.27, 1.0], 12.0),
        };

        let folder = if let Some(idx) = file_path.rfind('/') {
            file_path[..idx].to_string()
        } else {
            "root".to_string()
        };

        let stereotype = match kind {
            SymbolKind::Function => "«Function»".to_string(),
            SymbolKind::File => "«File»".to_string(),
            SymbolKind::ClassOrStruct => "«Class»".to_string(),
            SymbolKind::FormOrEndpoint => "«Endpoint»".to_string(),
            SymbolKind::Contract => "«Contract»".to_string(),
        };

        let symbol_type = match kind {
            SymbolKind::Function => "function".to_string(),
            SymbolKind::File => "file".to_string(),
            SymbolKind::ClassOrStruct => "class".to_string(),
            SymbolKind::FormOrEndpoint => "endpoint".to_string(),
            SymbolKind::Contract => "contract".to_string(),
        };

        let side = galaxy.clone();

        Self {
            id,
            name,
            file_path,
            folder,
            line_number,
            kind,
            galaxy,
            layer,
            stereotype,
            symbol_type,
            docstring: String::new(),
            is_class: kind == SymbolKind::ClassOrStruct,
            is_file: kind == SymbolKind::File,
            side,
            in_degree: 0,
            out_degree: 0,
            stage_idx: 0,
            complexity: 1,
            lines_count: 1,
            is_dead_code: false,
            has_security_violation: false,
            color_rgba: color,
            radius,
            x: 0.0,
            y: 0.0,
            birth_time: None,
            layouts: LayoutPositions::default(),
            metadata: SymbolMetadata::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyNode {
    pub id: String,
    pub name: String,
    pub role: String,
    pub file_count: usize,
    pub symbol_count: usize,
    pub center: Point2D,
    pub is_remote: bool,
}
