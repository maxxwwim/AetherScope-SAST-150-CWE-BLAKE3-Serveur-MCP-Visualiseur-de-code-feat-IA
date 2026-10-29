use core_graph::graph::AetherGraph;

pub const INDEX_HTML: &str = include_str!("../assets/index.html");
pub const STYLES_CSS: &str = include_str!("../assets/styles.css");
pub const STATE_API_JS: &str = include_str!("../assets/state_api.js");
pub const WEBGL_SHADERS_PHYSICS_JS: &str = include_str!("../assets/webgl_shaders_physics.js");
pub const WEBGL_RENDERER_JS: &str = include_str!("../assets/webgl_renderer.js");
pub const UI_NAVIGATION_JS: &str = include_str!("../assets/ui_navigation.js");
pub const UI_INSPECTOR_DOCS_JS: &str = include_str!("../assets/ui_inspector_docs.js");
pub const UI_VISION_MODES_JS: &str = include_str!("../assets/ui_vision_modes.js");
pub const UI_AI_MODALS_JS: &str = include_str!("../assets/ui_ai_modals.js");

pub fn generate_standalone_html(graph: &AetherGraph) -> String {
    let json_data = serde_json::to_string(graph).unwrap_or_default();
    let injection = format!("<script>window.__AETHERSCOPE_DATA__ = {};</script>", json_data);

    let combined_js = format!(
        "<script>\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n</script>",
        STATE_API_JS,
        WEBGL_SHADERS_PHYSICS_JS,
        WEBGL_RENDERER_JS,
        UI_NAVIGATION_JS,
        UI_INSPECTOR_DOCS_JS,
        UI_VISION_MODES_JS,
        UI_AI_MODALS_JS
    );

    let html = INDEX_HTML
        .replace(
            "<link rel=\"stylesheet\" href=\"styles.css\" />",
            &format!("<style>\n{}\n</style>", STYLES_CSS),
        )
        .replace("<script src=\"state_api.js\"></script>", &combined_js)
        .replace("<script src=\"webgl_shaders_physics.js\"></script>", "")
        .replace("<script src=\"webgl_renderer.js\"></script>", "")
        .replace("<script src=\"ui_navigation.js\"></script>", "")
        .replace("<script src=\"ui_inspector_docs.js\"></script>", "")
        .replace("<script src=\"ui_vision_modes.js\"></script>", "")
        .replace("<script src=\"ui_ai_modals.js\"></script>", "");

    html.replace("<body", &format!("{}\n<body", injection))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_standalone_html() {
        let graph = AetherGraph::new("TestProject");
        let html = generate_standalone_html(&graph);

        assert!(html.contains("window.__AETHERSCOPE_DATA__"));
        assert!(html.contains("AetherAPI"));
        assert!(html.contains("aether-canvas"));
        assert!(!html.contains("href=\"styles.css\""));
        assert!(!html.contains("src=\"state_api.js\""));
        assert!(!html.contains("src=\"webgl_shaders_physics.js\""));
        assert!(!html.contains("src=\"webgl_renderer.js\""));
        assert!(!html.contains("src=\"ui_navigation.js\""));
        assert!(!html.contains("src=\"ui_inspector_docs.js\""));
        assert!(!html.contains("src=\"ui_vision_modes.js\""));
        assert!(!html.contains("src=\"ui_ai_modals.js\""));
        assert!(html.len() > 100_000);
    }
}
