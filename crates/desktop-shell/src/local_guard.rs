//! Garde d'accès du serveur Axum local.
//!
//! Le serveur écoute uniquement sur `127.0.0.1`, mais cela ne suffit pas : n'importe quelle page web
//! ouverte dans le navigateur peut émettre des requêtes vers `http://127.0.0.1:<port>` (CSRF), et une
//! attaque par *DNS rebinding* peut faire passer un nom de domaine externe pour le serveur local.
//!
//! Ce garde refuse donc toute requête dont :
//! - l'en-tête `Host` n'est pas une adresse de bouclage portant le port du serveur (anti DNS rebinding) ;
//! - l'en-tête `Origin` (s'il est présent) n'est pas le serveur lui-même (anti CSRF / appels cross-origin) ;
//! - l'en-tête `Sec-Fetch-Site` (s'il est présent) indique `cross-site` ou `same-site`.
//!
//! Le cockpit (WebView), le navigateur ouvert directement sur le serveur et le pont MCP (client TCP qui
//! envoie `Host: 127.0.0.1:<port>` sans `Origin`) restent acceptés.

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

#[derive(Clone, Debug)]
pub struct LocalGuard {
    port: u16,
}

impl LocalGuard {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    fn authorities(&self) -> [String; 3] {
        [
            format!("127.0.0.1:{}", self.port),
            format!("localhost:{}", self.port),
            format!("[::1]:{}", self.port),
        ]
    }

    /// Décision pure (testable) : la requête est-elle légitime pour ce serveur local ?
    pub fn is_allowed(
        &self,
        host: Option<&str>,
        origin: Option<&str>,
        sec_fetch_site: Option<&str>,
    ) -> bool {
        let authorities = self.authorities();

        let host_ok = match host {
            Some(h) => authorities.iter().any(|a| a.eq_ignore_ascii_case(h.trim())),
            None => false,
        };
        if !host_ok {
            return false;
        }

        if let Some(o) = origin {
            let o = o.trim();
            let origin_ok = authorities
                .iter()
                .any(|a| o.eq_ignore_ascii_case(&format!("http://{}", a)));
            if !origin_ok {
                return false;
            }
        }

        if let Some(site) = sec_fetch_site {
            let site = site.trim().to_ascii_lowercase();
            if site != "same-origin" && site != "none" {
                return false;
            }
        }

        true
    }
}

/// Middleware Axum appliquant [`LocalGuard`] à chaque requête.
pub async fn enforce_local_only(
    State(guard): State<LocalGuard>,
    req: Request,
    next: Next,
) -> Response {
    let allowed = {
        let header_str = |name: &str| req.headers().get(name).and_then(|v| v.to_str().ok());
        guard.is_allowed(
            header_str(header::HOST.as_str()),
            header_str(header::ORIGIN.as_str()),
            header_str("sec-fetch-site"),
        )
    };

    if allowed {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            "AetherScope: requête refusée (origine non locale)",
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> LocalGuard {
        LocalGuard::new(3030)
    }

    #[test]
    fn accepts_mcp_bridge_without_origin() {
        assert!(guard().is_allowed(Some("127.0.0.1:3030"), None, None));
    }

    #[test]
    fn accepts_cockpit_same_origin_post() {
        assert!(guard().is_allowed(
            Some("127.0.0.1:3030"),
            Some("http://127.0.0.1:3030"),
            Some("same-origin")
        ));
        assert!(guard().is_allowed(
            Some("localhost:3030"),
            Some("http://localhost:3030"),
            Some("same-origin")
        ));
    }

    #[test]
    fn accepts_user_typed_navigation() {
        assert!(guard().is_allowed(Some("127.0.0.1:3030"), None, Some("none")));
    }

    #[test]
    fn rejects_dns_rebinding_host() {
        assert!(!guard().is_allowed(Some("evil.example.com:3030"), None, None));
        assert!(!guard().is_allowed(Some("evil.example.com"), None, None));
    }

    #[test]
    fn rejects_missing_or_wrong_port_host() {
        assert!(!guard().is_allowed(None, None, None));
        assert!(!guard().is_allowed(Some("127.0.0.1:4040"), None, None));
        assert!(!guard().is_allowed(Some("127.0.0.1"), None, None));
    }

    #[test]
    fn rejects_cross_origin_requests() {
        assert!(!guard().is_allowed(
            Some("127.0.0.1:3030"),
            Some("https://evil.example.com"),
            None
        ));
        assert!(!guard().is_allowed(Some("127.0.0.1:3030"), Some("null"), None));
    }

    #[test]
    fn rejects_cross_site_fetch_metadata() {
        assert!(!guard().is_allowed(Some("127.0.0.1:3030"), None, Some("cross-site")));
        assert!(!guard().is_allowed(Some("127.0.0.1:3030"), None, Some("same-site")));
    }
}
