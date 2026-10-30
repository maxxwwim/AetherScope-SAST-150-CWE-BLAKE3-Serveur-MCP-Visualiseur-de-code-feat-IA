use crate::linter::{CveRuleDef, LinterViolation};

pub struct PhpSecurityRules;

impl PhpSecurityRules {
    /// Détermine si un chemin correspond à un script ou module PHP
    pub fn is_php_file(file_path: &str) -> bool {
        let p = file_path.to_lowercase();
        p.ends_with(".php")
            || p.ends_with(".phtml")
            || p.ends_with(".php3")
            || p.ends_with(".php4")
            || p.ends_with(".php5")
            || p.ends_with(".php7")
            || p.ends_with(".phps")
    }

    /// Catalogue exhaustif des 25 règles de sécurité PHP CRITIQUES & HAUTES
    pub fn definitions() -> Vec<CveRuleDef> {
        vec![
            // 1. CWE-89: SQL Injection
            CveRuleDef {
                rule_id: "CVE-PHP-SQL-INJECTION".to_string(),
                cve_id: "CWE-89 / SQL-INJECTION".to_string(),
                name: "Injection SQL Directe via Concaténation de Variables Externes".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Concaténation directe de données d'entrée ($_GET, $_POST, $_REQUEST, $_COOKIE, variables non assainies) dans une requête SQL (PDO::query, PDO::exec, mysqli_query, mysql_query) sans requête préparée ($pdo->prepare() avec bindParam/execute([$param])) : permet l'exfiltration ou la destruction totale de la base de données.".to_string(),
                default_enabled: true,
            },
            // 2. CWE-78: OS Command Injection
            CveRuleDef {
                rule_id: "CVE-PHP-COMMAND-INJECTION".to_string(),
                cve_id: "CWE-78 / OS-COMMAND-INJECTION".to_string(),
                name: "Injection de Commande Système OS Arbitraire (RCE)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de commandes système OS via system(), exec(), passthru(), shell_exec(), proc_open(), popen() ou backticks recevant des variables externes sans validation ni échappement obligatoire via escapeshellarg() ou escapeshellcmd().".to_string(),
                default_enabled: true,
            },
            // 3. CWE-94: Code Injection / Eval RCE
            CveRuleDef {
                rule_id: "CVE-PHP-EVAL-CODE-INJECTION".to_string(),
                cve_id: "CWE-94 / EVAL-CODE-INJECTION".to_string(),
                name: "Exécution de Code PHP Dynamique Arbitraire (eval / create_function)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Évaluation dynamique de chaînes comme code PHP via eval(), create_function(), assert() dynamique ou preg_replace() avec modificateur /e : prise de contrôle totale du serveur web via exécution de code arbitraire à distance (RCE).".to_string(),
                default_enabled: true,
            },
            // 4. CWE-502: Insecure Deserialization (Object Injection)
            CveRuleDef {
                rule_id: "CVE-PHP-UNSERIALIZE-RCE".to_string(),
                cve_id: "CWE-502 / OBJECT-INJECTION-RCE".to_string(),
                name: "Désérialisation Non Sécurisée d'Objets PHP (POP Gadget Chain RCE)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Désérialisation d'un flux externe via unserialize() sans restriction ['allowed_classes' => false] : instanciation malveillante de classes et exécution automatique des méthodes magiques (__destruct, __wakeup) menant à un RCE.".to_string(),
                default_enabled: true,
            },
            // 5. CWE-98: Dynamic File Inclusion (RFI / LFI)
            CveRuleDef {
                rule_id: "CVE-PHP-FILE-INCLUSION-RFI-LFI".to_string(),
                cve_id: "CWE-98 / DYNAMIC-FILE-INCLUSION".to_string(),
                name: "Inclusion Dynamique de Fichier Local ou Distant (LFI / RFI)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Inclusion dynamique de script via include, include_once, require, require_once alimentée par des entrées utilisateur ($_GET, $_POST, variables non filtrées) sans liste blanche fermée, permettant l'inclusion de code malveillant ou wrappers php://filter.".to_string(),
                default_enabled: true,
            },
            // 6. CWE-22: Path Traversal / Arbitrary File Access
            CveRuleDef {
                rule_id: "CVE-PHP-PATH-TRAVERSAL".to_string(),
                cve_id: "CWE-22 / ARBITRARY-FILE-ACCESS".to_string(),
                name: "Traversée de Répertoire & Manipulation Arbitraire de Fichier (Path Traversal)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Accès, lecture ou suppression de fichiers système hors du répertoire autorisé via file_get_contents(), readfile(), file_put_contents(), unlink() ou fopen() avec des entrées utilisateur sans appel à basename() ni vérification de préfixe realpath().".to_string(),
                default_enabled: true,
            },
            // 7. CWE-79: Cross-Site Scripting (Reflected / Stored XSS)
            CveRuleDef {
                rule_id: "CVE-PHP-XSS-UNESCAPED-OUTPUT".to_string(),
                cve_id: "CWE-79 / REFLECTED-STORED-XSS".to_string(),
                name: "Affichage Non Échappé de Données Utilisateur (Cross-Site Scripting XSS)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Affichage direct dans le document HTTP (echo, print, <?=) de superglobales ($_GET, $_POST, $_REQUEST, $_SERVER['PHP_SELF']) sans échappement contextuel via htmlspecialchars($var, ENT_QUOTES, 'UTF-8') : vol de cookies de session et défacement.".to_string(),
                default_enabled: true,
            },
            // 8. CWE-918: Server-Side Request Forgery (SSRF)
            CveRuleDef {
                rule_id: "CVE-PHP-SSRF-CURL-STREAM".to_string(),
                cve_id: "CWE-918 / SERVER-SIDE-REQUEST-FORGERY".to_string(),
                name: "Requête Réseau Sortante Vers Hôte Non Filtré (SSRF)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Émission de requêtes HTTP sortantes via curl_exec(), file_get_contents($url) ou fopen($url) vers une URL fournie par le client sans filtrage des adresses IP locales et privées (127.0.0.1, 169.254.169.254, RFC 1918) ni blocage des protocoles gopher/file.".to_string(),
                default_enabled: true,
            },
            // 9. CWE-434: Unrestricted File Upload
            CveRuleDef {
                rule_id: "CVE-PHP-UNRESTRICTED-FILE-UPLOAD".to_string(),
                cve_id: "CWE-434 / UNRESTRICTED-FILE-UPLOAD".to_string(),
                name: "Téléversement de Fichier Sans Restriction d'Extension (Webshell Upload)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Déplacement de fichier téléversé via move_uploaded_file() en conservant directement le nom d'origine client ($_FILES['...']['name']) sans vérification stricte par liste blanche d'extensions ni génération de nom aléatoire haché : upload direct de webshells .php.".to_string(),
                default_enabled: true,
            },
            // 10. CWE-611: XML External Entity (XXE) Injection
            CveRuleDef {
                rule_id: "CVE-PHP-XXE-LIBXML-INJECTION".to_string(),
                cve_id: "CWE-611 / XML-EXTERNAL-ENTITY-INJECTION".to_string(),
                name: "Parsing XML Sans Désactivation des Entités Externes (XXE)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Analyse de flux XML via simplexml_load_string(), simplexml_load_file() ou DOMDocument::loadXML() avec entités externes activées (LIBXML_NOENT sans LIBXML_NONET ou sans libxml_disable_entity_loader(true)) : fuite de fichiers locaux et scan de ports internes.".to_string(),
                default_enabled: true,
            },
            // 11. CWE-384 / CWE-287: Session Fixation
            CveRuleDef {
                rule_id: "CVE-PHP-SESSION-FIXATION-NO-REGEN".to_string(),
                cve_id: "CWE-384 / SESSION-FIXATION".to_string(),
                name: "Affectation d'Authentification en Session Sans Régénération d'ID".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Élévation de privilèges ou enregistrement d'état authentifié dans $_SESSION['user_id'] ou $_SESSION['auth'] sans appel préalable à session_regenerate_id(true), permettant à un attaquant ayant fixé l'identifiant de session de détourner le compte.".to_string(),
                default_enabled: true,
            },
            // 12. CWE-312 / CWE-319: Insecure Cookie Flags
            CveRuleDef {
                rule_id: "CVE-PHP-COOKIE-SENSITIVE-PLAINTEXT".to_string(),
                cve_id: "CWE-312 / INSECURE-COOKIE-TRANSMISSION".to_string(),
                name: "Émission de Cookie Sensible Sans Flags Secure / HttpOnly".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Création de cookies d'authentification ou tokens de session via setcookie() ou session_set_cookie_params() avec drapeaux secure ou httponly à false ou omis, autorisant l'accès par scripts tiers (XSS) et la transmission en clair sur HTTP non chiffré.".to_string(),
                default_enabled: true,
            },
            // 13. CWE-362: File Race Condition (TOCTOU)
            CveRuleDef {
                rule_id: "CVE-PHP-FILE-LOCKING-TOCTOU".to_string(),
                cve_id: "CWE-362 / TOCTOU-FILE-RACE-CONDITION".to_string(),
                name: "Condition de Course TOCTOU sur Fichier Sensible Sans Verrou Atomique".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Vérification préalable d'un fichier (file_exists / is_file) immédiatement suivie d'une écriture sans verrou exclusif flock($fp, LOCK_EX) ni mode atomique 'x' : fenêtre de concurrence temporelle permettant l'écrasement ou l'altération de ressource critique.".to_string(),
                default_enabled: true,
            },
            // 14. CWE-327: Broken Crypto MD5 / SHA1
            CveRuleDef {
                rule_id: "CVE-PHP-BROKEN-CRYPTO-MD5-SHA1".to_string(),
                cve_id: "CWE-327 / BROKEN-CRYPTOGRAPHIC-HASH".to_string(),
                name: "Hachage Cryptographique Obsolète pour Mots de Passe ou Secrets (MD5 / SHA1)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Utilisation d'algorithmes de hachage rapides obsolètes (md5(), sha1(), crypt()) pour le traitement de mots de passe, clés secrètes ou jetons critiques au lieu de fonctions modernes résistantes aux attaques GPU (password_hash(..., PASSWORD_ARGON2ID / PASSWORD_BCRYPT)).".to_string(),
                default_enabled: true,
            },
            // 15. CWE-113: HTTP Header Injection / CRLF
            CveRuleDef {
                rule_id: "CVE-PHP-HEADER-INJECTION-CRLF".to_string(),
                cve_id: "CWE-113 / HTTP-RESPONSE-SPLITTING".to_string(),
                name: "Injection d'En-têtes HTTP (CRLF Injection & Open Redirect)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Émission d'en-têtes HTTP via header() incluant des données utilisateur ($_GET, $_POST, $target) sans assainissement des sauts de ligne (\\r, \\n) ni validation d'URL cible : permet le découpage de réponse HTTP (Response Splitting) ou la redirection ouverte vers des domaines d'hameçonnage.".to_string(),
                default_enabled: true,
            },
            // 16. CWE-621 / CWE-471: Variable Overwrite via extract() / Variable Variables
            CveRuleDef {
                rule_id: "CVE-PHP-VARIABLE-OVERWRITE-EXTRACT".to_string(),
                cve_id: "CWE-621 / VARIABLE-OVERWRITE-EXTRACT".to_string(),
                name: "Écrasement Arbitraire de Variables via extract() ou Variables Dynamiques".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Importation non sécurisée de variables externes via extract($_GET), extract($_POST), extract($_REQUEST) sans l'indicateur de sécurité EXTR_SKIP ou via variables dynamiques ($$var = $_GET[...]) : permet d'écraser des variables internes critiques ($is_admin, $role, $auth, identifiants DB) et de court-circuiter les contrôles d'accès.".to_string(),
                default_enabled: true,
            },
            // 17. CWE-88 / CWE-77: Mail Parameter Injection (Sendmail 5th Argument RCE)
            CveRuleDef {
                rule_id: "CVE-PHP-MAIL-PARAM-INJECTION".to_string(),
                cve_id: "CWE-88 / MAIL-PARAMETER-INJECTION".to_string(),
                name: "Injection de Paramètres Sendmail via mail() (5ème Argument RCE)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Utilisation de la fonction native mail() avec un 5ème paramètre ($additional_params) dynamique recevant des variables externes non protégées : permet à un attaquant d'injecter des drapeaux sendmail (-X pour écrire un webshell, -OQueueDirectory) menant à un Remote Code Execution (RCE).".to_string(),
                default_enabled: true,
            },
            // 18. CWE-1336 / CWE-94: Server-Side Template Injection (SSTI)
            CveRuleDef {
                rule_id: "CVE-PHP-TEMPLATE-INJECTION-SSTI".to_string(),
                cve_id: "CWE-1336 / SERVER-SIDE-TEMPLATE-INJECTION".to_string(),
                name: "Injection de Modèle Côté Serveur (SSTI dans Twig / Blade / Smarty)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Compilation ou rendu dynamique de modèles de templates (Twig::createTemplate, Blade::render, Smarty::display('string:...')) à partir de chaînes alimentées par des données utilisateur : permet l'exécution de code arbitraire à distance (RCE) via les filtres et fonctions internes du moteur de template.".to_string(),
                default_enabled: true,
            },
            // 19. CWE-295: Disabled SSL/TLS Verification
            CveRuleDef {
                rule_id: "CVE-PHP-DISABLED-SSL-VERIFY".to_string(),
                cve_id: "CWE-295 / DISABLED-SSL-VERIFICATION".to_string(),
                name: "Désactivation de la Vérification des Certificats SSL/TLS".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Désactivation explicite du contrôle de validité des certificats TLS/SSL via CURLOPT_SSL_VERIFYPEER à false, CURLOPT_SSL_VERIFYHOST à 0 ou contexte de stream 'verify_peer' => false : expose toutes les communications réseau sortantes (webhooks, APIs tierces, passerelles de paiement) aux attaques de type Man-in-the-Middle (MitM).".to_string(),
                default_enabled: true,
            },
            // 20. CWE-697 / CWE-472: Loose Comparison & Type Juggling in Auth/Hashes
            CveRuleDef {
                rule_id: "CVE-PHP-TYPE-JUGGLING-AUTH".to_string(),
                cve_id: "CWE-697 / LOOSE-COMPARISON-TYPE-JUGGLING".to_string(),
                name: "Comparaison Faible (Type Juggling) sur Secret, Token ou Mot de Passe".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Utilisation de comparaisons non strictes (==, !=, strcmp(...) == 0) lors de la validation d'un mot de passe, jeton HMAC, secret ou signature cryptographique : vulnérable au type juggling (notamment 'magic hashes' type 0e... où des hachages distincts sont considérés égaux) et aux attaques temporelles (timing attacks).".to_string(),
                default_enabled: true,
            },
            // 21. CWE-338: Cryptographically Weak PRNG for Tokens / Secrets
            CveRuleDef {
                rule_id: "CVE-PHP-WEAK-PRNG-SECRETS".to_string(),
                cve_id: "CWE-338 / CRYPTOGRAPHICALLY-WEAK-PRNG".to_string(),
                name: "Générateur Aléatoire Non Cryptographique pour Secrets ou Jetons".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Génération d'identifiants de sécurité (tokens CSRF, jetons de réinitialisation de mot de passe, clés d'API, sessions) à l'aide de générateurs pseudo-aléatoires prédictibles (rand(), mt_rand(), uniqid(), str_shuffle()) : un attaquant peut reconstituer la graine et anticiper les futurs jetons émis.".to_string(),
                default_enabled: true,
            },
            // 22. CWE-90: LDAP Injection
            CveRuleDef {
                rule_id: "CVE-PHP-LDAP-INJECTION".to_string(),
                cve_id: "CWE-90 / LDAP-INJECTION".to_string(),
                name: "Injection LDAP Directe via Concaténation de Filtres".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Construction dynamique de filtres d'annuaire LDAP via ldap_search(), ldap_list() ou ldap_read() avec concaténation directe d'entrées utilisateur sans neutralisation : permet de court-circuiter l'authentification ou d'extraire l'annuaire d'entreprise complet.".to_string(),
                default_enabled: true,
            },
            // 23. CWE-943: NoSQL Injection (MongoDB BSON Operators)
            CveRuleDef {
                rule_id: "CVE-PHP-NOSQL-INJECTION".to_string(),
                cve_id: "CWE-943 / NOSQL-INJECTION-MONGODB".to_string(),
                name: "Injection d'Opérateurs NoSQL MongoDB via Tableaux Non Castés".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Passage direct de données d'entrée ($_POST, $_GET, $_REQUEST) sous forme de critères de recherche dans des requêtes MongoDB ($collection->find(), $collection->findOne(), $collection->count()) sans conversion explicite en chaîne : injection d'opérateurs BSON (ex: ['$ne' => null]) permettant de court-circuiter l'authentification.".to_string(),
                default_enabled: true,
            },
            // 24. CWE-770 / CWE-22: Zip Slip Archive Extraction
            CveRuleDef {
                rule_id: "CVE-PHP-ZIP-SLIP-EXTRACTION".to_string(),
                cve_id: "CWE-770 / ZIP-SLIP-ARBITRARY-EXTRACTION".to_string(),
                name: "Décompression d'Archive Non Sécurisée (Vulnérabilité Zip Slip)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Extraction d'archives compressées via ZipArchive::extractTo() sans vérification de confinement des chemins relatifs (séquences ../) : un fichier d'archive malveillant peut écraser des fichiers de code exécutable arbitraires hors du dossier cible.".to_string(),
                default_enabled: true,
            },
            // 25. CWE-209 / CWE-215: Exposed Debug Mode & Sensitive Diagnostic Info
            CveRuleDef {
                rule_id: "CVE-PHP-EXPOSED-DEBUG-CONFIG".to_string(),
                cve_id: "CWE-209 / SENSITIVE-INFO-DEBUG-EXPOSURE".to_string(),
                name: "Exposition de Configuration de Débogage & Messages d'Erreur (phpinfo / display_errors)".to_string(),
                category: "PHP / Web Security".to_string(),
                severity: "HIGH".to_string(),
                description: "Activation explicite de l'affichage des erreurs à l'utilisateur (ini_set('display_errors', '1')) ou appel à la fonction de diagnostic phpinfo() dans un script web accessible : fuite critique d'informations d'infrastructure, de variables d'environnement ($_ENV), de chemins serveurs et de versions logicielles.".to_string(),
                default_enabled: true,
            },
        ]
    }

    /// Audit statique approfondi d'un fichier PHP
    pub fn audit_file(
        file_path: &str,
        content: &str,
        is_test_file: bool,
        is_interface_or_types: bool,
    ) -> Vec<LinterViolation> {
        let mut violations = Vec::new();

        // Règle d'or : Les règles PHP ne doivent s'exécuter STRICTEMENT QUE sur des fichiers PHP
        if !Self::is_php_file(file_path) || is_test_file || is_interface_or_types {
            return violations;
        }

        let lines: Vec<&str> = content.lines().collect();
        let has_session_regen = content.contains("session_regenerate_id");
        let has_xml_entity_loader_disabled = content.contains("libxml_disable_entity_loader(true)")
            || content.contains("LIBXML_NONET");
        let has_flock = content.contains("flock(") && (content.contains("LOCK_EX") || content.contains("LOCK_SH"));

        for (idx, &line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with('*') {
                continue;
            }

            // -------------------------------------------------------------
            // 1. CVE-PHP-SQL-INJECTION (CWE-89)
            // -------------------------------------------------------------
            let is_sql_call = trimmed.contains("mysqli_query(")
                || trimmed.contains("mysql_query(")
                || trimmed.contains("->query(")
                || trimmed.contains("->exec(")
                || (trimmed.contains("->prepare(") && (trimmed.contains(" . $") || trimmed.contains(".$") || trimmed.contains("{$")));

            let has_tainted_sql_concat = trimmed.contains("$_GET")
                || trimmed.contains("$_POST")
                || trimmed.contains("$_REQUEST")
                || trimmed.contains("$_COOKIE")
                || ((trimmed.contains("SELECT ") || trimmed.contains("INSERT INTO ") || trimmed.contains("UPDATE ") || trimmed.contains("DELETE FROM ") || trimmed.contains("WHERE "))
                    && (trimmed.contains(" . $") || trimmed.contains(".$") || trimmed.contains(" = '\" .") || trimmed.contains(" = '\".")));

            if is_sql_call && has_tainted_sql_concat && !trimmed.contains("bind_param") && !trimmed.contains("bindParam") && !trimmed.contains("bindValue") {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-SQL-INJECTION".to_string(),
                    cve_id: "CWE-89 / SQL-INJECTION".to_string(),
                    severity: "CRITICAL".to_string(),
                    message: "🚨 [CWE-89] SQL-INJECTION : Requête SQL exécutée via concaténation directe d'entrées externes sans requête préparée ($pdo->prepare avec execute([$id]) ou bind_param). Permet le vol complet de la base de données.".to_string(),
                    transmitted_data: Some("raw_sql_query_concat".to_string()),
                    transmitted_type: Some("String (Unsanitized Dynamic SQL)".to_string()),
                    remediation: "Utiliser une requête préparée paramétrée : $stmt = $pdo->prepare('SELECT * FROM users WHERE id = :id'); $stmt->execute(['id' => $id]);".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 2. CVE-PHP-COMMAND-INJECTION (CWE-78)
            // -------------------------------------------------------------
            let is_shell_exec = trimmed.contains("system(")
                || trimmed.contains("exec(")
                || trimmed.contains("passthru(")
                || trimmed.contains("shell_exec(")
                || trimmed.contains("proc_open(")
                || trimmed.contains("popen(")
                || (trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.contains('$'));

            if is_shell_exec {
                let has_var_or_input = trimmed.contains("$_GET")
                    || trimmed.contains("$_POST")
                    || trimmed.contains("$_REQUEST")
                    || trimmed.contains(" . $")
                    || trimmed.contains(".$")
                    || (trimmed.contains('$') && !trimmed.contains("escapeshellarg") && !trimmed.contains("escapeshellcmd"));

                let is_escaped = trimmed.contains("escapeshellarg(") || trimmed.contains("escapeshellcmd(");

                if has_var_or_input && !is_escaped {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-COMMAND-INJECTION".to_string(),
                        cve_id: "CWE-78 / OS-COMMAND-INJECTION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "🚨 [CWE-78] OS-COMMAND-INJECTION : Exécution de commandes shell OS avec des paramètres dynamiques non assainis. Permet l'exécution de code système arbitraire (RCE) via des métacaractères (; & |).".to_string(),
                        transmitted_data: Some("shell_exec_command".to_string()),
                        transmitted_type: Some("String (Unsanitized OS Command)".to_string()),
                        remediation: "Échapper chaque paramètre avec escapeshellarg($arg) ou utiliser des fonctions PHP natives sans shell.".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 3. CVE-PHP-EVAL-CODE-INJECTION (CWE-94)
            // -------------------------------------------------------------
            let is_eval = trimmed.contains("eval(")
                || trimmed.contains("create_function(")
                || (trimmed.contains("assert(") && (trimmed.contains("$_GET") || trimmed.contains("$_POST") || trimmed.contains(" . $")))
                || (trimmed.contains("preg_replace(") && (trimmed.contains("/e'") || trimmed.contains("/e\"") || trimmed.contains("'/e'") || trimmed.contains("\"/e\"")));

            if is_eval {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-EVAL-CODE-INJECTION".to_string(),
                    cve_id: "CWE-94 / EVAL-CODE-INJECTION".to_string(),
                    severity: "CRITICAL".to_string(),
                    message: "🚨 [CWE-94] EVAL-CODE-INJECTION : Évaluation dynamique de code PHP via eval(), create_function() ou assert() dynamique. Prise de contrôle totale du serveur par Remote Code Execution (RCE).".to_string(),
                    transmitted_data: Some("eval_code_string".to_string()),
                    transmitted_type: Some("String (Dynamic PHP Code)".to_string()),
                    remediation: "Supprimer eval() et create_function(). Remplacer preg_replace('/.../e') par preg_replace_callback() et utiliser des parsers stricts pour les expressions logiques.".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 4. CVE-PHP-UNSERIALIZE-RCE (CWE-502)
            // -------------------------------------------------------------
            if trimmed.contains("unserialize(") {
                let has_allowed_false = trimmed.contains("'allowed_classes' => false")
                    || trimmed.contains("\"allowed_classes\" => false")
                    || trimmed.contains("'allowed_classes'=>false")
                    || trimmed.contains("\"allowed_classes\"=>false");

                if !has_allowed_false {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-UNSERIALIZE-RCE".to_string(),
                        cve_id: "CWE-502 / OBJECT-INJECTION-RCE".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "🚨 [CWE-502] OBJECT-INJECTION-RCE : Désérialisation d'objets via unserialize() sans l'option ['allowed_classes' => false]. Déclenche des chaînes de gadgets POP (__destruct, __wakeup) menant à un RCE.".to_string(),
                        transmitted_data: Some("unserialize_raw_data".to_string()),
                        transmitted_type: Some("String (Untrusted Serialized Stream)".to_string()),
                        remediation: "Utiliser json_decode($json, true) ou restreindre strictement : unserialize($input, ['allowed_classes' => false]).".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 5. CVE-PHP-FILE-INCLUSION-RFI-LFI (CWE-98)
            // -------------------------------------------------------------
            let is_include_stmt = (trimmed.starts_with("include ") || trimmed.starts_with("include_once ")
                || trimmed.starts_with("require ") || trimmed.starts_with("require_once ")
                || trimmed.starts_with("include(") || trimmed.starts_with("require("))
                && !trimmed.contains("__DIR__")
                && !trimmed.contains("vendor/autoload.php");

            if is_include_stmt {
                let is_dynamic = trimmed.contains("$_GET")
                    || trimmed.contains("$_POST")
                    || trimmed.contains("$_REQUEST")
                    || (trimmed.contains('$') && (trimmed.contains(" . ") || trimmed.contains(".$")));

                let has_whitelist = trimmed.contains("in_array(")
                    || trimmed.contains("match (")
                    || trimmed.contains("switch (")
                    || content.contains("in_array(")
                    || content.contains("ALLOWED_")
                    || content.contains("whitelist");

                if is_dynamic && !has_whitelist {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-FILE-INCLUSION-RFI-LFI".to_string(),
                        cve_id: "CWE-98 / DYNAMIC-FILE-INCLUSION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "🚨 [CWE-98] DYNAMIC-FILE-INCLUSION : Inclusion dynamique de script PHP via include/require recevant une variable externe non filtrée. Risque d'exécution de code arbitraire LFI/RFI et wrappers php://filter.".to_string(),
                        transmitted_data: Some("include_file_target".to_string()),
                        transmitted_type: Some("String (Dynamic Inclusion Target)".to_string()),
                        remediation: "Vérifier le paramètre dans une liste blanche statique avant inclusion : if (in_array($page, ALLOWED_PAGES, true)) { require $page . '.php'; }.".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 6. CVE-PHP-PATH-TRAVERSAL (CWE-22)
            // -------------------------------------------------------------
            let is_file_access = trimmed.contains("file_get_contents(")
                || trimmed.contains("readfile(")
                || trimmed.contains("file_put_contents(")
                || trimmed.contains("unlink(")
                || (trimmed.contains("fopen(") && !trimmed.contains("php://"));

            if is_file_access {
                let has_tainted_file = trimmed.contains("$_GET")
                    || trimmed.contains("$_POST")
                    || trimmed.contains("$_REQUEST")
                    || (trimmed.contains('$') && (trimmed.contains("uploads/") || trimmed.contains("files/") || trimmed.contains("data/")));

                let has_traversal_guard = trimmed.contains("basename(")
                    || trimmed.contains("realpath(")
                    || content.contains("basename(")
                    || content.contains("realpath(");

                if has_tainted_file && !has_traversal_guard {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-PATH-TRAVERSAL".to_string(),
                        cve_id: "CWE-22 / ARBITRARY-FILE-ACCESS".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-22] ARBITRARY-FILE-ACCESS : Manipulation de fichier avec chemin externe sans neutralisation des séquences de traversée (../). Risque d'accès en lecture/écriture à des fichiers hors racine.".to_string(),
                        transmitted_data: Some("file_path_expression".to_string()),
                        transmitted_type: Some("String (Path Traversal Vulnerable Path)".to_string()),
                        remediation: "Appliquer basename($filename) pour éliminer les répertoires ou vérifier str_starts_with(realpath($path), realpath($allowed_dir)).".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 7. CVE-PHP-XSS-UNESCAPED-OUTPUT (CWE-79)
            // -------------------------------------------------------------
            let is_output = trimmed.starts_with("echo ")
                || trimmed.starts_with("print ")
                || trimmed.starts_with("printf(")
                || trimmed.starts_with("<?=");

            if is_output {
                let has_raw_user_input = trimmed.contains("$_GET[")
                    || trimmed.contains("$_POST[")
                    || trimmed.contains("$_REQUEST[")
                    || trimmed.contains("$_SERVER['PHP_SELF']")
                    || trimmed.contains("$_SERVER[\"PHP_SELF\"]")
                    || trimmed.contains("$_SERVER['REQUEST_URI']")
                    || trimmed.contains("$_SERVER[\"REQUEST_URI\"]");

                let has_xss_protection = trimmed.contains("htmlspecialchars(")
                    || trimmed.contains("htmlentities(")
                    || trimmed.contains("urlencode(")
                    || trimmed.contains("intval(")
                    || trimmed.contains("(int)");

                if has_raw_user_input && !has_xss_protection {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-XSS-UNESCAPED-OUTPUT".to_string(),
                        cve_id: "CWE-79 / REFLECTED-STORED-XSS".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-79] REFLECTED-STORED-XSS : Donnée externe superglobale émise directement dans la réponse HTTP sans échappement HTML. Permet l'injection de scripts malveillants et le vol de session.".to_string(),
                        transmitted_data: Some("raw_xss_echo_payload".to_string()),
                        transmitted_type: Some("String (Unescaped HTML Stream)".to_string()),
                        remediation: "Échapper systématiquement les données avec htmlspecialchars($str, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8').".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 8. CVE-PHP-SSRF-CURL-STREAM (CWE-918)
            // -------------------------------------------------------------
            let is_curl_or_stream_req = (trimmed.contains("CURLOPT_URL") && trimmed.contains('$'))
                || (trimmed.contains("file_get_contents(") && (trimmed.contains("$_GET") || trimmed.contains("$_POST") || trimmed.contains("$url")))
                || (trimmed.contains("fopen(") && (trimmed.contains("$_GET") || trimmed.contains("$_POST") || trimmed.contains("$url")));

            if is_curl_or_stream_req {
                let has_ip_filter = content.contains("FILTER_FLAG_NO_PRIV_RANGE")
                    || content.contains("FILTER_FLAG_NO_RES_RANGE")
                    || content.contains("check_private_ip")
                    || content.contains("is_private_ip");

                if !has_ip_filter {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-SSRF-CURL-STREAM".to_string(),
                        cve_id: "CWE-918 / SERVER-SIDE-REQUEST-FORGERY".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-918] SERVER-SIDE-REQUEST-FORGERY : Requête réseau sortante vers une URL dynamique sans vérification des plages d'IPs privées/locales (127.0.0.1, 169.254.169.254). Permet le scan interne et le vol de secrets cloud.".to_string(),
                        transmitted_data: Some("outbound_http_url".to_string()),
                        transmitted_type: Some("String (Unvalidated SSRF Endpoint)".to_string()),
                        remediation: "Résoudre l'adresse IP et valider filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE) en interdisant les redirections vers des IP locales.".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 9. CVE-PHP-UNRESTRICTED-FILE-UPLOAD (CWE-434)
            // -------------------------------------------------------------
            if trimmed.contains("move_uploaded_file(") {
                let uses_client_name = trimmed.contains("['name']") || trimmed.contains("[\"name\"]");
                let has_random_rename = trimmed.contains("random_bytes")
                    || trimmed.contains("bin2hex")
                    || trimmed.contains("uniqid(")
                    || trimmed.contains("sha256");

                if uses_client_name && !has_random_rename {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-UNRESTRICTED-FILE-UPLOAD".to_string(),
                        cve_id: "CWE-434 / UNRESTRICTED-FILE-UPLOAD".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "🚨 [CWE-434] UNRESTRICTED-FILE-UPLOAD : Téléversement de fichier conservant directement le nom d'origine client sans renommage aléatoire sécurisé ni liste blanche stricte. Risque d'upload de webshell exécutable .php.".to_string(),
                        transmitted_data: Some("uploaded_file_destination".to_string()),
                        transmitted_type: Some("String (Client Supplied Upload Filename)".to_string()),
                        remediation: "Valider l'extension avec une liste blanche stricte (.jpg, .png, .pdf) et générer un nom aléatoire unique : bin2hex(random_bytes(16)) . '.' . $ext;".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 10. CVE-PHP-XXE-LIBXML-INJECTION (CWE-611)
            // -------------------------------------------------------------
            let is_xml_parse = trimmed.contains("simplexml_load_string(")
                || trimmed.contains("simplexml_load_file(")
                || trimmed.contains("->loadXML(");

            if is_xml_parse {
                let is_vulnerable = trimmed.contains("LIBXML_NOENT") || (!has_xml_entity_loader_disabled && !trimmed.contains("LIBXML_NONET"));
                if is_vulnerable {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-XXE-LIBXML-INJECTION".to_string(),
                        cve_id: "CWE-611 / XML-EXTERNAL-ENTITY-INJECTION".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-611] XML-EXTERNAL-ENTITY-INJECTION : Analyse de document XML avec entités externes potentiellement activées sans LIBXML_NONET ni désactivation du chargeur d'entités. Permet la lecture de fichiers système.".to_string(),
                        transmitted_data: Some("xml_content_stream".to_string()),
                        transmitted_type: Some("String (Untrusted XML Document)".to_string()),
                        remediation: "Désactiver les entités externes avec libxml_disable_entity_loader(true) ou passer l'option LIBXML_NONET lors de l'appel au parser.".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 11. CVE-PHP-SESSION-FIXATION-NO-REGEN (CWE-384 / CWE-287)
            // -------------------------------------------------------------
            let is_session_auth_write = (trimmed.contains("$_SESSION['user_id']") || trimmed.contains("$_SESSION[\"user_id\"]")
                || trimmed.contains("$_SESSION['auth']") || trimmed.contains("$_SESSION[\"auth\"]")
                || trimmed.contains("$_SESSION['logged_in']") || trimmed.contains("$_SESSION[\"logged_in\"]"))
                && trimmed.contains('=');

            if is_session_auth_write && !has_session_regen {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-SESSION-FIXATION-NO-REGEN".to_string(),
                    cve_id: "CWE-384 / SESSION-FIXATION".to_string(),
                    severity: "HIGH".to_string(),
                    message: "⚠️ [CWE-384] SESSION-FIXATION : Modification des droits d'authentification en session sans régénération préalable de l'identifiant (session_regenerate_id). Permet le détournement de compte par session fixation.".to_string(),
                    transmitted_data: Some("session_auth_elevation".to_string()),
                    transmitted_type: Some("Session (Unregenerated Session ID)".to_string()),
                    remediation: "Appeler session_regenerate_id(true) immédiatement lors de la validation des identifiants avant d'écrire en session.".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 12. CVE-PHP-COOKIE-SENSITIVE-PLAINTEXT (CWE-312 / CWE-319)
            // -------------------------------------------------------------
            if trimmed.contains("setcookie(") {
                let is_auth_cookie = trimmed.contains("auth")
                    || trimmed.contains("token")
                    || trimmed.contains("session")
                    || trimmed.contains("jwt")
                    || trimmed.contains("user");

                let is_missing_security_flags = (trimmed.contains(", false") && trimmed.matches(", false").count() >= 1)
                    || (!trimmed.contains("'secure'") && !trimmed.contains("\"secure\"") && !trimmed.contains(", true"));

                if is_auth_cookie && is_missing_security_flags {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-COOKIE-SENSITIVE-PLAINTEXT".to_string(),
                        cve_id: "CWE-312 / INSECURE-COOKIE-TRANSMISSION".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-312] INSECURE-COOKIE-TRANSMISSION : Cookie sensible ou jeton d'authentification configuré sans les drapeaux Secure ou HttpOnly. Permet l'interception sur réseau en clair et l'accès via document.cookie (XSS).".to_string(),
                        transmitted_data: Some("auth_cookie_options".to_string()),
                        transmitted_type: Some("CookieOptions (Insecure Flags)".to_string()),
                        remediation: "Définir impérativement ['secure' => true, 'httponly' => true, 'samesite' => 'Strict'] dans les options du cookie.".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 13. CVE-PHP-FILE-LOCKING-TOCTOU (CWE-362)
            // -------------------------------------------------------------
            let is_file_check = (trimmed.contains("file_exists(") || trimmed.contains("is_file("))
                && (content.contains("file_put_contents(") || content.contains("fopen("))
                && !content.contains("LOCK_EX")
                && !content.contains("'x'");

            if is_file_check && !has_flock {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-FILE-LOCKING-TOCTOU".to_string(),
                    cve_id: "CWE-362 / TOCTOU-FILE-RACE-CONDITION".to_string(),
                    severity: "HIGH".to_string(),
                    message: "⚠️ [CWE-362] TOCTOU-FILE-RACE-CONDITION : Vérification d'existence de fichier suivie d'une écriture sans verrou exclusif flock($fp, LOCK_EX) ni mode de création atomique ('x'). Vulnérable aux conditions de course temporelles.".to_string(),
                    transmitted_data: Some("file_toctou_handle".to_string()),
                    transmitted_type: Some("FileAccess (Unsynchronized File Write)".to_string()),
                    remediation: "Ouvrir avec le flag atomique fopen($file, 'x') ou acquérir un verrou exclusif flock($fp, LOCK_EX) avant manipulation.".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 14. CVE-PHP-BROKEN-CRYPTO-MD5-SHA1 (CWE-327)
            // -------------------------------------------------------------
            let is_weak_hash = trimmed.contains("md5(") || trimmed.contains("sha1(") || trimmed.contains("crypt(");
            let is_password_or_secret = trimmed.contains("pass")
                || trimmed.contains("secret")
                || trimmed.contains("token")
                || trimmed.contains("pwd");

            if is_weak_hash && is_password_or_secret && !content.contains("password_hash") {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-BROKEN-CRYPTO-MD5-SHA1".to_string(),
                    cve_id: "CWE-327 / BROKEN-CRYPTOGRAPHIC-HASH".to_string(),
                    severity: "HIGH".to_string(),
                    message: "⚠️ [CWE-327] BROKEN-CRYPTOGRAPHIC-HASH : Algorithme de hachage cassé (MD5/SHA1/crypt) utilisé pour traiter des mots de passe ou secrets. Vulnérable aux collisions et aux attaques par force brute GPU.".to_string(),
                    transmitted_data: Some("weak_hash_algorithm".to_string()),
                    transmitted_type: Some("CryptoHash (Weak MD5/SHA1 Hash)".to_string()),
                    remediation: "Remplacer par l'API moderne password_hash($password, PASSWORD_ARGON2ID) ou PASSWORD_BCRYPT.".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 15. CVE-PHP-HEADER-INJECTION-CRLF (CWE-113 / CWE-74)
            // -------------------------------------------------------------
            if trimmed.contains("header(") {
                let is_redirect_or_custom = trimmed.contains("Location:") || trimmed.contains("X-");
                let has_user_param = trimmed.contains("$_GET") || trimmed.contains("$_POST") || trimmed.contains("$url") || trimmed.contains("$target");
                let has_crlf_strip = trimmed.contains("str_replace") && (trimmed.contains("\\r") || trimmed.contains("\\n") || trimmed.contains("\r") || trimmed.contains("\n"));
                let has_url_validate = trimmed.contains("FILTER_VALIDATE_URL");

                if is_redirect_or_custom && has_user_param && !has_crlf_strip && !has_url_validate {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-HEADER-INJECTION-CRLF".to_string(),
                        cve_id: "CWE-113 / HTTP-RESPONSE-SPLITTING".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-113] HTTP-RESPONSE-SPLITTING : En-tête HTTP Location ou personnalisé alimenté par une entrée externe sans assainissement des sauts de ligne (\\r\\n). Risque d'injection CRLF et redirection ouverte (Open Redirect).".to_string(),
                        transmitted_data: Some("header_redirect_url".to_string()),
                        transmitted_type: Some("String (Unsanitized HTTP Response Header)".to_string()),
                        remediation: "Supprimer les caractères CRLF via str_replace([\"\\r\", \"\\n\"], '', $url) et valider l'URL avec filter_var($url, FILTER_VALIDATE_URL).".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 16. CVE-PHP-VARIABLE-OVERWRITE-EXTRACT (CWE-621 / CWE-471)
            // -------------------------------------------------------------
            let is_extract_call = trimmed.contains("extract(");
            let has_extract_untrusted = trimmed.contains("$_GET")
                || trimmed.contains("$_POST")
                || trimmed.contains("$_REQUEST")
                || trimmed.contains("$_COOKIE")
                || trimmed.contains("$request")
                || trimmed.contains("$data");
            let has_extract_safe_flag = trimmed.contains("EXTR_SKIP")
                || trimmed.contains("EXTR_PREFIX_ALL")
                || trimmed.contains("EXTR_PREFIX_INVALID")
                || trimmed.contains("EXTR_PREFIX_SAME");

            let is_dynamic_var_overwrite = trimmed.contains("$$")
                && (trimmed.contains("$_GET") || trimmed.contains("$_POST") || trimmed.contains("$_REQUEST"));

            if (is_extract_call && has_extract_untrusted && !has_extract_safe_flag) || is_dynamic_var_overwrite {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-VARIABLE-OVERWRITE-EXTRACT".to_string(),
                    cve_id: "CWE-621 / VARIABLE-OVERWRITE-EXTRACT".to_string(),
                    severity: "CRITICAL".to_string(),
                    message: "🚨 [CWE-621] VARIABLE-OVERWRITE-EXTRACT : Écrasement arbitraire de variables de portée locale via extract() sans EXTR_SKIP ou assignation dynamique $$. Permet à un attaquant de court-circuiter l'authentification en modifiant des variables critiques ($is_admin, $role).".to_string(),
                    transmitted_data: Some("untrusted_extracted_array".to_string()),
                    transmitted_type: Some("Array (Arbitrary Variable Overwrite)".to_string()),
                    remediation: "Ne jamais appeler extract() sur des entrées utilisateur. Si indispensable, forcer le drapeau EXTR_SKIP : extract($_GET, EXTR_SKIP); ou manipuler directement le tableau associatif.".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 17. CVE-PHP-MAIL-PARAM-INJECTION (CWE-88 / CWE-77)
            // -------------------------------------------------------------
            let is_native_mail = (trimmed.contains("mail(") || trimmed.contains("mail ("))
                && !trimmed.contains("wp_mail(")
                && !trimmed.contains("_mail(")
                && !trimmed.contains("->mail(");

            if is_native_mail {
                let parts: Vec<&str> = trimmed.split(',').collect();
                if parts.len() >= 5 {
                    let fifth_arg = parts[4..].join(",");
                    let has_var_in_fifth = fifth_arg.contains('$');
                    let is_escaped = fifth_arg.contains("escapeshellcmd(") || fifth_arg.contains("escapeshellarg(");

                    if has_var_in_fifth && !is_escaped {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "CVE-PHP-MAIL-PARAM-INJECTION".to_string(),
                            cve_id: "CWE-88 / MAIL-PARAMETER-INJECTION".to_string(),
                            severity: "CRITICAL".to_string(),
                            message: "🚨 [CWE-88] MAIL-PARAMETER-INJECTION : Le 5ème paramètre de mail() est alimenté dynamiquement sans escapeshellcmd(). Permet l'injection de drapeaux MTA (-X /var/www/shell.php) menant à un Remote Code Execution (RCE).".to_string(),
                            transmitted_data: Some("mail_additional_params".to_string()),
                            transmitted_type: Some("String (Unsanitized Sendmail Parameter)".to_string()),
                            remediation: "Échapper systématiquement le 5ème paramètre avec escapeshellcmd($extra) ou utiliser une bibliothèque SMTP moderne (Symfony Mailer).".to_string(),
                        });
                    }
                }
            }

            // -------------------------------------------------------------
            // 18. CVE-PHP-TEMPLATE-INJECTION-SSTI (CWE-1336 / CWE-94)
            // -------------------------------------------------------------
            let is_template_compile = trimmed.contains("->createTemplate(")
                || trimmed.contains("Blade::render(")
                || (trimmed.contains("->display(") && trimmed.contains("string:"))
                || (trimmed.contains("->render(") && (trimmed.contains("template_from_string") || trimmed.contains("inline:")));

            let has_tainted_template_code = trimmed.contains("$_GET")
                || trimmed.contains("$_POST")
                || trimmed.contains("$_REQUEST")
                || trimmed.contains("$input")
                || trimmed.contains("$template")
                || trimmed.contains("$tpl")
                || trimmed.contains("$code");

            if is_template_compile && has_tainted_template_code {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-TEMPLATE-INJECTION-SSTI".to_string(),
                    cve_id: "CWE-1336 / SERVER-SIDE-TEMPLATE-INJECTION".to_string(),
                    severity: "CRITICAL".to_string(),
                    message: "🚨 [CWE-1336] SERVER-SIDE-TEMPLATE-INJECTION : Rendu ou instanciation dynamique de modèle de template (Twig/Blade/Smarty) avec une chaîne utilisateur. Permet l'exécution de code arbitraire (RCE) via les filtres du moteur de templates.".to_string(),
                    transmitted_data: Some("dynamic_template_source".to_string()),
                    transmitted_type: Some("String (Untrusted Template Code)".to_string()),
                    remediation: "Ne jamais compiler de chaîne utilisateur en modèle. Charger un template statique et transmettre les données en arguments de contexte : $twig->render('page.html.twig', ['data' => $data]);".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 19. CVE-PHP-DISABLED-SSL-VERIFY (CWE-295)
            // -------------------------------------------------------------
            let is_curl_ssl_disabled = (trimmed.contains("CURLOPT_SSL_VERIFYPEER") && (trimmed.contains("false") || trimmed.contains(", 0") || trimmed.contains(",0")))
                || (trimmed.contains("CURLOPT_SSL_VERIFYHOST") && (trimmed.contains(", 0") || trimmed.contains(",0")));

            let is_stream_ssl_disabled = (trimmed.contains("'verify_peer' => false") || trimmed.contains("\"verify_peer\" => false") || trimmed.contains("'verify_peer'=>false") || trimmed.contains("\"verify_peer\"=>false"))
                || (trimmed.contains("'verify_peer_name' => false") || trimmed.contains("\"verify_peer_name\" => false") || trimmed.contains("'verify_peer_name'=>false") || trimmed.contains("\"verify_peer_name\"=>false"));

            if is_curl_ssl_disabled || is_stream_ssl_disabled {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-DISABLED-SSL-VERIFY".to_string(),
                    cve_id: "CWE-295 / DISABLED-SSL-VERIFICATION".to_string(),
                    severity: "HIGH".to_string(),
                    message: "⚠️ [CWE-295] DISABLED-SSL-VERIFICATION : Désactivation de la vérification des certificats SSL/TLS (CURLOPT_SSL_VERIFYPEER false ou verify_peer false). Les requêtes sortantes vers des APIs ou passerelles de paiement sont vulnérables au Man-in-the-Middle (MitM).".to_string(),
                    transmitted_data: Some("insecure_ssl_context_option".to_string()),
                    transmitted_type: Some("SSLConfig (Disabled Certificate Check)".to_string()),
                    remediation: "Maintenir CURLOPT_SSL_VERIFYPEER à true et CURLOPT_SSL_VERIFYHOST à 2. Spécifier un certificat CA racine valide avec CURLOPT_CAINFO si nécessaire.".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 20. CVE-PHP-TYPE-JUGGLING-AUTH (CWE-697 / CWE-472)
            // -------------------------------------------------------------
            let has_loose_cmp = {
                let bytes = trimmed.as_bytes();
                let mut found = false;
                let mut i = 0;
                while i + 1 < bytes.len() {
                    if bytes[i] == b'=' && bytes[i + 1] == b'=' {
                        let prev_char = if i > 0 { Some(bytes[i - 1]) } else { None };
                        let next_char = if i + 2 < bytes.len() { Some(bytes[i + 2]) } else { None };
                        let is_triple = prev_char == Some(b'=') || next_char == Some(b'=');
                        let is_strict_ineq = prev_char == Some(b'!');
                        if !is_triple && !is_strict_ineq {
                            found = true;
                            break;
                        }
                    }
                    i += 1;
                }
                found || (trimmed.contains("strcmp(") && (trimmed.contains("== 0") || trimmed.contains("==0")))
            };

            let is_auth_sensitive = trimmed.contains("password")
                || trimmed.contains("token")
                || trimmed.contains("hash")
                || trimmed.contains("secret")
                || trimmed.contains("signature")
                || trimmed.contains("auth")
                || trimmed.contains("apiKey")
                || trimmed.contains("api_key");

            if has_loose_cmp && is_auth_sensitive && !trimmed.contains("hash_equals(") {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-TYPE-JUGGLING-AUTH".to_string(),
                    cve_id: "CWE-697 / LOOSE-COMPARISON-TYPE-JUGGLING".to_string(),
                    severity: "HIGH".to_string(),
                    message: "⚠️ [CWE-697] LOOSE-COMPARISON-TYPE-JUGGLING : Comparaison non stricte (== ou strcmp == 0) sur un secret, mot de passe ou jeton sensible. Vulnérable au Type Juggling ('magic hashes' 0e...) et aux attaques par canal auxiliaire (timing attacks).".to_string(),
                    transmitted_data: Some("loose_equality_expression".to_string()),
                    transmitted_type: Some("BooleanExpression (Loose Type Comparison)".to_string()),
                    remediation: "Utiliser la fonction à temps constant hash_equals($secret, $input) ou l'opérateur d'égalité stricte ===.".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 21. CVE-PHP-WEAK-PRNG-SECRETS (CWE-338)
            // -------------------------------------------------------------
            let is_weak_prng = trimmed.contains("rand(")
                || trimmed.contains("mt_rand(")
                || trimmed.contains("uniqid(")
                || trimmed.contains("str_shuffle(");

            let is_security_token_context = trimmed.contains("token")
                || trimmed.contains("secret")
                || trimmed.contains("csrf")
                || trimmed.contains("key")
                || trimmed.contains("password")
                || trimmed.contains("reset")
                || trimmed.contains("auth_key");

            if is_weak_prng && is_security_token_context && !trimmed.contains("random_bytes") && !trimmed.contains("random_int") {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-WEAK-PRNG-SECRETS".to_string(),
                    cve_id: "CWE-338 / CRYPTOGRAPHICALLY-WEAK-PRNG".to_string(),
                    severity: "HIGH".to_string(),
                    message: "⚠️ [CWE-338] CRYPTOGRAPHICALLY-WEAK-PRNG : Générateur pseudo-aléatoire non cryptographique (rand, mt_rand, uniqid) employé pour générer un jeton ou secret sensible. La graine est prédictible par un attaquant.".to_string(),
                    transmitted_data: Some("weak_prng_seed_output".to_string()),
                    transmitted_type: Some("RandomToken (Weak Pseudo-Random)".to_string()),
                    remediation: "Employer les fonctions CSPRNG natives : bin2hex(random_bytes(32)) ou random_int($min, $max).".to_string(),
                });
            }

            // -------------------------------------------------------------
            // 22. CVE-PHP-LDAP-INJECTION (CWE-90)
            // -------------------------------------------------------------
            let is_ldap_query = trimmed.contains("ldap_search(")
                || trimmed.contains("ldap_list(")
                || trimmed.contains("ldap_read(");

            if is_ldap_query {
                let has_tainted_ldap = trimmed.contains("$_GET")
                    || trimmed.contains("$_POST")
                    || trimmed.contains("$_REQUEST")
                    || (trimmed.contains('$') && (trimmed.contains(" . $") || trimmed.contains(".$") || trimmed.contains("{$")));

                let is_ldap_escaped = trimmed.contains("ldap_escape(") || content.contains("ldap_escape(");

                if has_tainted_ldap && !is_ldap_escaped {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-LDAP-INJECTION".to_string(),
                        cve_id: "CWE-90 / LDAP-INJECTION".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-90] LDAP-INJECTION : Filtre d'annuaire LDAP alimenté par des données utilisateur sans échappement ldap_escape(). Permet de court-circuiter l'authentification d'entreprise et d'exfiltrer l'annuaire.".to_string(),
                        transmitted_data: Some("ldap_filter_string".to_string()),
                        transmitted_type: Some("String (Unsanitized LDAP Filter)".to_string()),
                        remediation: "Échapper chaque paramètre du filtre avec ldap_escape($input, null, LDAP_ESCAPE_FILTER).".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 23. CVE-PHP-NOSQL-INJECTION (CWE-943)
            // -------------------------------------------------------------
            let is_mongodb_find = trimmed.contains("->find(")
                || trimmed.contains("->findOne(")
                || trimmed.contains("->countDocuments(")
                || trimmed.contains("->count(");

            if is_mongodb_find {
                let has_raw_post_input = trimmed.contains("$_POST[")
                    || trimmed.contains("$_GET[")
                    || trimmed.contains("$_REQUEST[");

                let has_type_cast = trimmed.contains("(string)") || trimmed.contains("strval(");

                if has_raw_post_input && !has_type_cast {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-NOSQL-INJECTION".to_string(),
                        cve_id: "CWE-943 / NOSQL-INJECTION-MONGODB".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-943] NOSQL-INJECTION-MONGODB : Requête MongoDB alimentée directement par un champ superglobal sans transtypage strict (string). Permet l'injection d'opérateurs BSON ($ne, $gt) et le bypass d'authentification.".to_string(),
                        transmitted_data: Some("mongodb_query_document".to_string()),
                        transmitted_type: Some("Array (Uncast BSON Criteria)".to_string()),
                        remediation: "Transtyper obligatoirement en chaîne : ['password' => (string)$_POST['password']] ou valider le type avec is_string().".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 24. CVE-PHP-ZIP-SLIP-EXTRACTION (CWE-770 / CWE-22)
            // -------------------------------------------------------------
            if trimmed.contains("->extractTo(") {
                let has_path_containment_check = content.contains("realpath(")
                    || content.contains("str_starts_with(")
                    || content.contains("substr(")
                    || content.contains("strpos(");

                if !has_path_containment_check {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PHP-ZIP-SLIP-EXTRACTION".to_string(),
                        cve_id: "CWE-770 / ZIP-SLIP-ARBITRARY-EXTRACTION".to_string(),
                        severity: "HIGH".to_string(),
                        message: "⚠️ [CWE-770] ZIP-SLIP-ARBITRARY-EXTRACTION : Décompression d'archive via ZipArchive::extractTo() sans validation de confinement des entrées. Un ZIP malveillant contenant des séquences ../ peut écraser du code système.".to_string(),
                        transmitted_data: Some("zip_archive_target_directory".to_string()),
                        transmitted_type: Some("Path (Unvalidated Archive Extraction)".to_string()),
                        remediation: "Inspecter chaque entrée avec $zip->getNameIndex($i), résoudre realpath() et s'assurer que le chemin extrait commence strictement par le répertoire cible autorisé.".to_string(),
                    });
                }
            }

            // -------------------------------------------------------------
            // 25. CVE-PHP-EXPOSED-DEBUG-CONFIG (CWE-209 / CWE-215)
            // -------------------------------------------------------------
            let is_display_errors_on = (trimmed.contains("ini_set('display_errors'") || trimmed.contains("ini_set(\"display_errors\""))
                && (trimmed.contains("'1'") || trimmed.contains("\"1\"") || trimmed.contains(", 1)") || trimmed.contains(",1)"));

            let is_phpinfo_call = trimmed.contains("phpinfo()");

            if is_display_errors_on || is_phpinfo_call {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "CVE-PHP-EXPOSED-DEBUG-CONFIG".to_string(),
                    cve_id: "CWE-209 / SENSITIVE-INFO-DEBUG-EXPOSURE".to_string(),
                    severity: "HIGH".to_string(),
                    message: "⚠️ [CWE-209] SENSITIVE-INFO-DEBUG-EXPOSURE : Activation d'affichage des erreurs détaillées (display_errors) ou appel de diagnostic phpinfo(). Révèle les variables d'environnement, identifiants et arborescence serveur.".to_string(),
                    transmitted_data: Some("debug_configuration_state".to_string()),
                    transmitted_type: Some("Config (Exposed Diagnostic Interface)".to_string()),
                    remediation: "Désactiver l'affichage direct des erreurs en production avec ini_set('display_errors', '0'), journaliser via error_log() et supprimer tout appel à phpinfo().".to_string(),
                });
            }
        }

        violations
    }
}
