use core_graph::bubble::SymbolBubble;
use core_graph::edge::CategorizedLiaisons;
use security_linter::contract::ReadmeContract;

pub struct HandshakePacker;

impl HandshakePacker {
    pub fn build_prompt(
        action_type: &str, // "Refactor" | "Test" | "Audit" | "Doc" | "Target"
        symbol: &SymbolBubble,
        callers: &[String],
        dependencies: &[String],
        contract: Option<&ReadmeContract>,
    ) -> String {
        let mut in_cat = CategorizedLiaisons::default();
        in_cat.internal_calls = callers.to_vec();
        let mut out_cat = CategorizedLiaisons::default();
        out_cat.internal_calls = dependencies.to_vec();
        Self::build_categorized_prompt(action_type, symbol, &in_cat, &out_cat, contract)
    }

    pub fn build_categorized_prompt(
        action_type: &str,
        symbol: &SymbolBubble,
        incoming: &CategorizedLiaisons,
        outgoing: &CategorizedLiaisons,
        contract: Option<&ReadmeContract>,
    ) -> String {
        let mut xml = String::new();
        xml.push_str("<aether_handshake version=\"2.1\">\n");
        xml.push_str(&format!("  <action>{}</action>\n", action_type));
        xml.push_str("  <target>\n");
        xml.push_str(&format!("    <name>{}</name>\n", symbol.name));
        xml.push_str(&format!("    <file>{}:{}</file>\n", symbol.file_path, symbol.line_number));
        xml.push_str(&format!("    <galaxy>{}</galaxy>\n", symbol.galaxy));
        xml.push_str(&format!("    <layer>{}</layer>\n", symbol.layer));
        xml.push_str("  </target>\n");

        xml.push_str("  <context>\n");
        fn render_group(xml: &mut String, tag: &str, list: &[String]) {
            if !list.is_empty() {
                xml.push_str(&format!("      <{}_group count=\"{}\">\n", tag, list.len()));
                for item in list.iter().take(4) {
                    xml.push_str(&format!("        <{}>{}</{}>\n", tag, item.trim_matches('`'), tag));
                }
                if list.len() > 4 {
                    xml.push_str(&format!("        <more_hidden count=\"{}\" />\n", list.len() - 4));
                }
                xml.push_str(&format!("      </{}_group>\n", tag));
            }
        }

        render_group(&mut xml, "form_submit", &incoming.form_submits);
        render_group(&mut xml, "network_http", &incoming.network_http);
        render_group(&mut xml, "realtime_ws", &incoming.realtime_ws);
        render_group(&mut xml, "database_query", &incoming.database_queries);
        render_group(&mut xml, "caller", &incoming.internal_calls);
        render_group(&mut xml, "dependency", &incoming.external_deps);
        render_group(&mut xml, "violation", &incoming.violations);
        xml.push_str("    </in_degree>\n");

        xml.push_str(&format!("    <out_degree total=\"{}\">\n", outgoing.total()));
        render_group(&mut xml, "form_submit", &outgoing.form_submits);
        render_group(&mut xml, "network_http", &outgoing.network_http);
        render_group(&mut xml, "realtime_ws", &outgoing.realtime_ws);
        render_group(&mut xml, "database_query", &outgoing.database_queries);
        render_group(&mut xml, "caller", &outgoing.internal_calls);
        render_group(&mut xml, "dependency", &outgoing.external_deps);
        render_group(&mut xml, "violation", &outgoing.violations);
        xml.push_str("    </out_degree>\n");
        xml.push_str("  </context>\n");

        if let Some(c) = contract {
            xml.push_str("  <contract_rules>\n");
            xml.push_str(&format!("    <max_lines>{}</max_lines>\n", c.frontmatter.max_lines_per_file));
            xml.push_str("    <allowed_imports>\n");
            for a in &c.frontmatter.allowed_imports {
                xml.push_str(&format!("      <import>{}</import>\n", a));
            }
            xml.push_str("    </allowed_imports>\n");
            xml.push_str("    <forbidden_imports>\n");
            for f in &c.frontmatter.forbidden_imports {
                xml.push_str(&format!("      <forbidden>{}</forbidden>\n", f));
            }
            xml.push_str("    </forbidden_imports>\n");
            xml.push_str("  </contract_rules>\n");
        }

        xml.push_str("  <instruction>\n");
        xml.push_str("    Avant d'écrire le code, réponds obligatoirement avec ce bloc de confirmation :\n");
        xml.push_str("    [AETHER_ACK]\n");
        xml.push_str(&format!("    Cible : {}\n", symbol.name));
        xml.push_str("    Imports interdits pris en compte : OUI\n");
        xml.push_str("    Audit live surveillé dans .aether/LIVE_AUDIT.md : OUI\n");
        xml.push_str("    [/AETHER_ACK]\n");
        xml.push_str("  </instruction>\n");
        xml.push_str("</aether_handshake>\n");

        xml
    }
}

