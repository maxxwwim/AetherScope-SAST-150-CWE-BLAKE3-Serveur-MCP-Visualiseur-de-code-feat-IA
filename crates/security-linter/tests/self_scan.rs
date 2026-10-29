//! Auto-scan : le linter ne doit pas se signaler lui-même à cause de ses propres
//! descriptions de règles / catalogues de documentation (faux positifs réflexifs).
//! Lancer avec `--nocapture` pour voir le décompte par fichier et par règle.

use security_linter::SecurityLinter;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn scan(rel: &str) -> Vec<security_linter::LinterViolation> {
    let p = workspace_root().join(rel);
    let content = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {}", p.display(), e));
    SecurityLinter::audit_file(rel, &content, None)
}

#[test]
fn self_scan_report_and_reflexive_false_positives() {
    let files = [
        "crates/security-linter/src/linter.rs",
        "crates/security-linter/src/rules.rs",
        "crates/security-linter/src/contract.rs",
        "crates/desktop-shell/assets/ui_vision_modes.js",
        "crates/desktop-shell/assets/ui_ai_modals.js",
        "crates/desktop-shell/assets/ui_inspector_docs.js",
    ];
    let mut total = 0;
    for f in files {
        let v = scan(f);
        let mut by_rule: BTreeMap<String, usize> = BTreeMap::new();
        for x in &v {
            *by_rule.entry(x.rule_id.clone()).or_default() += 1;
        }
        println!("{:<55} {:>3} violations {:?}", f, v.len(), by_rule);
        total += v.len();
    }
    println!("TOTAL (6 fichiers): {}", total);

    // Le catalogue de documentation JS ne contient aucun appel réel à eval/exec/sendFile.
    // Limite connue : CVE-WEB3-AUTH-BYPASS-MOCK, SAST_HARDCODED_SECRET et CVE-JS-XSS-DOM
    // restent signalés (règles sensibles aux littéraux / innerHTML réels à trier à la main).
    let js = scan("crates/desktop-shell/assets/ui_vision_modes.js");
    for rule in [
        "CVE-JS-CODE-INJ",
        "CVE-JS-CMD-INJ",
        "CVE-JS-PATH-TRAVERSAL-SENDFILE",
    ] {
        assert!(
            !js.iter().any(|v| v.rule_id == rule),
            "faux positif réflexif {} dans ui_vision_modes.js",
            rule
        );
    }

    // Les définitions de règles (rules.rs) ne doivent déclencher aucune règle de code dangereux.
    let rules = scan("crates/security-linter/src/rules.rs");
    for v in &rules {
        assert!(
            !["CVE-RUST-UNSAFE-MEM", "CVE-RUST-CMD-INJ", "CVE-RUST-ASYNC-MUTEX"]
                .contains(&v.rule_id.as_str()),
            "faux positif réflexif {} ligne {} dans rules.rs",
            v.rule_id,
            v.line_number
        );
    }
}
