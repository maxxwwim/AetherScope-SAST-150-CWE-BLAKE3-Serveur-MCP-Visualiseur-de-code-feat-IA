use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct FilterAffiliation: u32 {
        const PHYSICAL          = 1 << 0; // Topology & Direct file imports
        const NETWORK_RPC       = 1 << 1; // Form/Fetch -> Server Route (Orange Laser)
        const REALTIME_WS       = 1 << 2; // WebSocket Packet / Web3 On-Chain (Cyan Laser)
        const SEMANTIC          = 1 << 3; // Role / Stereotype conventions
        const SECURITY_VIOLATES = 1 << 4; // README Guardrail Contract Violation (Red Blinking)
        const DATABASE_QUERY    = 1 << 5; // DB & SQL Persistence (Cyber Emerald Laser)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    Contains,
    Import,
    Call,
    Render,
    Inherit,
    Network,
    NetworkRPC,
    RealtimeWS,
    ContractBinding,
    IllegalCrossGalaxy,
    Documents,
    DatabaseQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliationEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    #[serde(default)]
    pub order_label: String,
    pub edge_type: EdgeType,
    pub filters: FilterAffiliation,
    #[serde(default)]
    pub filter_bits: u32,
    pub weight: f32,
    pub label: Option<String>,
    pub violation_details: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cve_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted_type: Option<String>,
}

impl AffiliationEdge {
    pub fn new(source: String, target: String, edge_type: EdgeType, filters: FilterAffiliation) -> Self {
        let relation = match edge_type {
            EdgeType::Contains => "contains".to_string(),
            EdgeType::Import => "imports".to_string(),
            EdgeType::Call => "calls".to_string(),
            EdgeType::Render => "renders".to_string(),
            EdgeType::Inherit => "inherits".to_string(),
            EdgeType::Network => "network".to_string(),
            EdgeType::NetworkRPC => "submits".to_string(),
            EdgeType::RealtimeWS => "realtime".to_string(),
            EdgeType::ContractBinding => "binds".to_string(),
            EdgeType::IllegalCrossGalaxy => "violates".to_string(),
            EdgeType::Documents => "documents".to_string(),
            EdgeType::DatabaseQuery => "queries".to_string(),
        };

        Self {
            source,
            target,
            relation,
            order_label: String::new(),
            edge_type,
            filters,
            filter_bits: filters.bits(),
            weight: 1.0,
            label: None,
            violation_details: None,
            cve_id: None,
            severity: None,
            transmitted_data: None,
            transmitted_type: None,
        }
    }

    pub fn new_rpc(source: String, target: String, url: &str) -> Self {
        let filters = FilterAffiliation::NETWORK_RPC | FilterAffiliation::PHYSICAL;
        Self {
            source,
            target,
            relation: "submits".to_string(),
            order_label: "RPC".to_string(),
            edge_type: EdgeType::NetworkRPC,
            filters,
            filter_bits: filters.bits(),
            weight: 2.0,
            label: Some(url.to_string()),
            violation_details: None,
            cve_id: None,
            severity: None,
            transmitted_data: None,
            transmitted_type: None,
        }
    }

    pub fn new_realtime(source: String, target: String, packet_name: &str) -> Self {
        let filters = FilterAffiliation::REALTIME_WS | FilterAffiliation::PHYSICAL;
        Self {
            source,
            target,
            relation: "realtime".to_string(),
            order_label: "WS".to_string(),
            edge_type: EdgeType::RealtimeWS,
            filters,
            filter_bits: filters.bits(),
            weight: 2.5,
            label: Some(packet_name.to_string()),
            violation_details: None,
            cve_id: None,
            severity: None,
            transmitted_data: None,
            transmitted_type: None,
        }
    }

    pub fn new_violation(source: String, target: String, reason: &str) -> Self {
        Self::new_cve_violation(source, target, reason, None, None, None, None)
    }

    pub fn new_cve_violation(
        source: String,
        target: String,
        reason: &str,
        cve_id: Option<String>,
        severity: Option<String>,
        transmitted_data: Option<String>,
        transmitted_type: Option<String>,
    ) -> Self {
        let filters = FilterAffiliation::SECURITY_VIOLATES | FilterAffiliation::PHYSICAL;
        let label = cve_id.clone().unwrap_or_else(|| "VIOLATION".to_string());
        Self {
            source,
            target,
            relation: "violates".to_string(),
            order_label: "ERR".to_string(),
            edge_type: EdgeType::IllegalCrossGalaxy,
            filters,
            filter_bits: filters.bits(),
            weight: 3.0,
            label: Some(label),
            violation_details: Some(reason.to_string()),
            cve_id,
            severity,
            transmitted_data,
            transmitted_type,
        }
    }
}

/// Catégories sémantiques de liaisons pour structurer et alléger les prompts IA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LiaisonCategory {
    FormSubmit,     // 📝 Soumissions formulaires / UI Actions
    NetworkHttp,    // 🌐 Appels HTTP / Endpoints REST
    RealtimeWs,     // ⚡ WebSocket & Temps Réel MMO / Web3
    DatabaseQuery,  // 🗄️ Requêtes SQL & Persistance DB
    InternalCall,   // 📦 Appels internes de modules / fonctions
    ExternalDep,    // 🏛️ Dépendances transverses / contrats
    Documentation,  // 📖 Documentation / Markdown
    SecurityAlert,  // 🚨 Violation de contrat / CVE
}

impl LiaisonCategory {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::FormSubmit => "📝",
            Self::NetworkHttp => "🌐",
            Self::RealtimeWs => "⚡",
            Self::DatabaseQuery => "🗄️",
            Self::InternalCall => "📦",
            Self::ExternalDep => "🏛️",
            Self::Documentation => "📖",
            Self::SecurityAlert => "🚨",
        }
    }

    pub fn title_fr(&self) -> &'static str {
        match self {
            Self::FormSubmit => "Actions UI / Soumissions Formulaire",
            Self::NetworkHttp => "Appels Réseau & Endpoints HTTP / REST",
            Self::RealtimeWs => "Flux Temps Réel & Packets WebSocket",
            Self::DatabaseQuery => "Persistance & Requêtes Base de Données",
            Self::InternalCall => "Appels Internes / Fonctions",
            Self::ExternalDep => "Dépendances Transverses & Contrats",
            Self::Documentation => "Spécifications & Documentation",
            Self::SecurityAlert => "Violations de Sécurité / Alertes CVE",
        }
    }
}

/// Structure regroupée et allégée des liaisons pour les prompts IA
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorizedLiaisons {
    #[serde(default)]
    pub form_submits: Vec<String>,
    #[serde(default)]
    pub network_http: Vec<String>,
    #[serde(default)]
    pub realtime_ws: Vec<String>,
    #[serde(default)]
    pub database_queries: Vec<String>,
    #[serde(default)]
    pub internal_calls: Vec<String>,
    #[serde(default)]
    pub external_deps: Vec<String>,
    #[serde(default)]
    pub violations: Vec<String>,
}

impl CategorizedLiaisons {
    /// Catégorise une arête entrante ou sortante
    pub fn classify_edge(edge: &AffiliationEdge) -> LiaisonCategory {
        if edge.relation == "violates" || edge.filters.contains(FilterAffiliation::SECURITY_VIOLATES) {
            return LiaisonCategory::SecurityAlert;
        }
        if edge.relation == "submits" {
            return LiaisonCategory::FormSubmit;
        }
        if edge.relation == "network" || edge.edge_type == EdgeType::Network || edge.edge_type == EdgeType::NetworkRPC {
            return LiaisonCategory::NetworkHttp;
        }
        if edge.relation == "realtime" || edge.edge_type == EdgeType::RealtimeWS {
            return LiaisonCategory::RealtimeWs;
        }
        if edge.relation == "queries" || edge.edge_type == EdgeType::DatabaseQuery {
            return LiaisonCategory::DatabaseQuery;
        }
        if edge.relation == "documents" {
            return LiaisonCategory::Documentation;
        }
        if edge.relation == "imports" || edge.relation == "inherits" {
            return LiaisonCategory::ExternalDep;
        }
        LiaisonCategory::InternalCall
    }

    /// Construit la synthèse catégorisée à partir d'arêtes relatives à un symbole
    pub fn from_edges(edges: &[&AffiliationEdge], target_id: &str, is_incoming: bool) -> Self {
        let mut res = Self::default();
        for &e in edges {
            let other = if is_incoming { &e.source } else { &e.target };
            if other == target_id || other.is_empty() {
                continue;
            }
            let label = if let Some(lbl) = &e.label {
                format!("`{}` ({})", other, lbl)
            } else if !e.order_label.is_empty() {
                format!("`{}` ({})", other, e.order_label)
            } else {
                format!("`{}`", other)
            };

            match Self::classify_edge(e) {
                LiaisonCategory::FormSubmit => {
                    if !res.form_submits.contains(&label) { res.form_submits.push(label); }
                }
                LiaisonCategory::NetworkHttp => {
                    if !res.network_http.contains(&label) { res.network_http.push(label); }
                }
                LiaisonCategory::RealtimeWs => {
                    if !res.realtime_ws.contains(&label) { res.realtime_ws.push(label); }
                }
                LiaisonCategory::DatabaseQuery => {
                    if !res.database_queries.contains(&label) { res.database_queries.push(label); }
                }
                LiaisonCategory::ExternalDep => {
                    if !res.external_deps.contains(&label) { res.external_deps.push(label); }
                }
                LiaisonCategory::SecurityAlert => {
                    if !res.violations.contains(&label) { res.violations.push(label); }
                }
                _ => {
                    if !res.internal_calls.contains(&label) { res.internal_calls.push(label); }
                }
            }
        }
        res
    }

    /// Total des liaisons enregistrées
    pub fn total(&self) -> usize {
        self.form_submits.len()
            + self.network_http.len()
            + self.realtime_ws.len()
            + self.database_queries.len()
            + self.internal_calls.len()
            + self.external_deps.len()
            + self.violations.len()
    }

    /// Rendu Markdown compact allégé pour l'IA (cap de N éléments par catégorie avec compteur)
    pub fn to_compact_markdown(&self, prefix: &str, max_per_category: usize) -> String {
        if self.total() == 0 {
            return format!("*(Aucun flux {} détecté)*\n", prefix.to_lowercase());
        }

        let mut out = String::new();

        let format_section = |title: &str, icon: &str, list: &[String]| -> Option<String> {
            if list.is_empty() {
                return None;
            }
            let mut sec = format!("- **{} {}** ({} total) :\n", icon, title, list.len());
            for item in list.iter().take(max_per_category) {
                sec.push_str(&format!("  - {}\n", item));
            }
            if list.len() > max_per_category {
                sec.push_str(&format!("  - *… +{} autre(s) non affiché(s) pour préserver le contexte IA*\n", list.len() - max_per_category));
            }
            Some(sec)
        };

        if let Some(s) = format_section("Actions UI / Submits", "📝", &self.form_submits) {
            out.push_str(&s);
        }
        if let Some(s) = format_section("Endpoints / Requêtes HTTP", "🌐", &self.network_http) {
            out.push_str(&s);
        }
        if let Some(s) = format_section("Temps Réel & WebSocket", "⚡", &self.realtime_ws) {
            out.push_str(&s);
        }
        if let Some(s) = format_section("Persistance DB / SQL", "🗄️", &self.database_queries) {
            out.push_str(&s);
        }
        if let Some(s) = format_section("Dépendances Transverses & Imports", "🏛️", &self.external_deps) {
            out.push_str(&s);
        }
        if let Some(s) = format_section("Appels Internes", "📦", &self.internal_calls) {
            out.push_str(&s);
        }
        if let Some(s) = format_section("Violations & Alertes", "🚨", &self.violations) {
            out.push_str(&s);
        }

        out
    }
}
