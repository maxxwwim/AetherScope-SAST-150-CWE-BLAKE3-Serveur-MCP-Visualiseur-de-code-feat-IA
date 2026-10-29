//! Lexer léger : masque le contenu des littéraux de chaîne et des commentaires
//! (remplacés par des espaces) tout en préservant les délimiteurs, les retours à la
//! ligne et donc les numéros de ligne. Permet aux règles heuristiques de ne plus
//! se déclencher sur des descriptions de règles, de la documentation ou des fixtures.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// Rust (raw strings, lifetimes, commentaires imbriqués)
    Rust,
    /// JS / TS / Solidity / C / C++ (`//`, `/* */`, `'`, `"`, backtick)
    CLike,
    /// Python (`#`, triples quotes)
    Python,
}

impl Lang {
    pub fn from_path(lower_path: &str) -> Option<Lang> {
        if lower_path.ends_with(".rs") {
            Some(Lang::Rust)
        } else if lower_path.ends_with(".py") {
            Some(Lang::Python)
        } else if [
            ".js", ".ts", ".tsx", ".jsx", ".mjs", ".cjs", ".sol", ".circom", ".c", ".cpp", ".cc", ".h", ".hpp",
        ]
        .iter()
        .any(|e| lower_path.ends_with(e))
        {
            Some(Lang::CLike)
        } else {
            None
        }
    }
}

fn blank(c: char) -> char {
    if c == '\n' || c == '\r' {
        c
    } else {
        ' '
    }
}

/// Retourne une copie de `src` où strings et commentaires sont remplacés par des espaces.
pub fn mask_literals(src: &str, lang: Lang) -> String {
    mask_impl(src, lang, true)
}

/// Variante qui ne masque que les commentaires et conserve le contenu des cha�nes
/// (pour les r�gles dont la d�tection porte sur des litt�raux, ex. `algorithms=["none"]`).
pub fn mask_comments_only(src: &str, lang: Lang) -> String {
    mask_impl(src, lang, false)
}

fn mask_impl(src: &str, lang: Lang, mask_strings: bool) -> String {
    let sb = |c: char| if mask_strings { blank(c) } else { c };
    let ch: Vec<char> = src.chars().collect();
    let n = ch.len();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;

    while i < n {
        let c = ch[i];
        let next = if i + 1 < n { ch[i + 1] } else { '\0' };

        // --- commentaires ligne ---
        if (lang != Lang::Python && c == '/' && next == '/') || (lang == Lang::Python && c == '#') {
            while i < n && ch[i] != '\n' {
                out.push(blank(ch[i]));
                i += 1;
            }
            continue;
        }

        // --- commentaires bloc ---
        if lang != Lang::Python && c == '/' && next == '*' {
            let mut depth = 0usize;
            while i < n {
                if ch[i] == '/' && i + 1 < n && ch[i + 1] == '*' {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if ch[i] == '*' && i + 1 < n && ch[i + 1] == '/' {
                    out.push_str("  ");
                    i += 2;
                    depth -= 1;
                    // Rust: imbriqués ; autres langages: pas d'imbrication
                    if depth == 0 || lang != Lang::Rust {
                        break;
                    }
                } else {
                    out.push(blank(ch[i]));
                    i += 1;
                }
            }
            continue;
        }

        // --- Rust raw strings: r"..", r#".."#, br#".."# ---
        if lang == Lang::Rust && (c == 'r' || (c == 'b' && next == 'r')) {
            let prev_ident = i > 0 && (ch[i - 1].is_alphanumeric() || ch[i - 1] == '_');
            let mut j = if c == 'b' { i + 2 } else { i + 1 };
            let mut hashes = 0;
            while j < n && ch[j] == '#' {
                hashes += 1;
                j += 1;
            }
            if !prev_ident && j < n && ch[j] == '"' {
                for k in i..=j {
                    out.push(ch[k]);
                }
                i = j + 1;
                while i < n {
                    if ch[i] == '"' && (0..hashes).all(|h| i + 1 + h < n && ch[i + 1 + h] == '#') {
                        for k in 0..=hashes {
                            out.push(ch[i + k]);
                        }
                        i += hashes + 1;
                        break;
                    }
                    out.push(sb(ch[i]));
                    i += 1;
                }
                continue;
            }
        }

        // --- Python triples quotes ---
        if lang == Lang::Python
            && (c == '"' || c == '\'')
            && next == c
            && i + 2 < n
            && ch[i + 2] == c
        {
            out.push_str(&String::from_iter([c, c, c]));
            i += 3;
            while i < n {
                if ch[i] == c && i + 2 < n && ch[i + 1] == c && ch[i + 2] == c {
                    out.push_str(&String::from_iter([c, c, c]));
                    i += 3;
                    break;
                }
                out.push(sb(ch[i]));
                i += 1;
            }
            continue;
        }

        // --- Rust: char literal vs lifetime ---
        if lang == Lang::Rust && c == '\'' {
            let is_char_lit = (next == '\\')
                || (i + 2 < n && ch[i + 2] == '\'' && next != '\'' && next != '\n');
            if is_char_lit {
                out.push('\'');
                i += 1;
                while i < n && ch[i] != '\'' && ch[i] != '\n' {
                    if ch[i] == '\\' && i + 1 < n {
                        out.push(' ');
                        i += 1;
                    }
                    out.push(blank(ch[i]));
                    i += 1;
                }
                if i < n && ch[i] == '\'' {
                    out.push('\'');
                    i += 1;
                }
            } else {
                out.push(c);
                i += 1;
            }
            continue;
        }

        // --- chaînes simples ---
        let is_quote = c == '"'
            || (lang != Lang::Rust && c == '\'')
            || (lang == Lang::CLike && c == '`');
        if is_quote {
            // Seuls `"` Rust et les backticks peuvent s'étendre sur plusieurs lignes
            let multiline = (lang == Lang::Rust && c == '"') || c == '`';
            out.push(c);
            i += 1;
            while i < n {
                if ch[i] == '\\' && i + 1 < n {
                    out.push(sb('\\'));
                    out.push(sb(ch[i + 1]));
                    i += 2;
                    continue;
                }
                if ch[i] == c {
                    out.push(c);
                    i += 1;
                    break;
                }
                if ch[i] == '\n' && !multiline {
                    break; // chaîne non terminée: ne pas avaler la suite du fichier
                }
                out.push(sb(ch[i]));
                i += 1;
            }
            continue;
        }

        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_line_structure() {
        let src = "let a = \"x\ny\"; // c\nlet b = 1;\n";
        let m = mask_literals(src, Lang::Rust);
        assert_eq!(m.lines().count(), src.lines().count());
        assert_eq!(m.chars().count(), src.chars().count());
    }

    #[test]
    fn masks_rust_strings_comments_and_raw() {
        let m = mask_literals(
            "unsafe_call(); let d = \"unsafe { x }\"; // unsafe {\nlet r = r#\"unsafe { \"y\" }\"#;",
            Lang::Rust,
        );
        assert!(m.contains("unsafe_call()"));
        assert!(!m.contains("unsafe {"));
    }

    #[test]
    fn rust_lifetimes_do_not_swallow_code() {
        let m = mask_literals("fn f<'a>(x: &'a str) { unsafe { g() } }", Lang::Rust);
        assert!(m.contains("unsafe { g() }"));
    }

    #[test]
    fn rust_char_literals() {
        let m = mask_literals("let q = '\"'; unsafe { g() }", Lang::Rust);
        assert!(m.contains("unsafe { g() }"));
    }

    #[test]
    fn js_catalog_string_masked_but_code_kept() {
        let m = mask_literals(
            "el.innerHTML = x; const s = { shield: 'use innerHTML = safe' };",
            Lang::CLike,
        );
        assert!(m.starts_with("el.innerHTML = x;"));
        assert_eq!(m.matches("innerHTML").count(), 1);
    }

    #[test]
    fn unterminated_js_string_stops_at_eol() {
        let m = mask_literals("var a = 'oops\neval(x)", Lang::CLike);
        assert!(m.contains("eval(x)"));
    }

    #[test]
    fn python_triple_quotes_and_hash() {
        let m = mask_literals("\"\"\"os.system(x)\"\"\"\nos.system(y) # os.system(z)\n", Lang::Python);
        assert_eq!(m.matches("os.system").count(), 1);
    }
}
