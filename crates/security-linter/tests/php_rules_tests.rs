use security_linter::{PhpSecurityRules, SecurityLinter};

#[test]
fn test_cwe_catalog_includes_25_php_rules_and_file_detection() {
    let catalog = SecurityLinter::get_supported_cve_rules();
    assert!(
        catalog.len() >= 129,
        "Expected at least 129 rules, got {}",
        catalog.len()
    );

    let php_rules: Vec<_> = catalog
        .iter()
        .filter(|r| r.category == "PHP / Web Security" || r.rule_id.starts_with("CVE-PHP-"))
        .collect();

    assert_eq!(
        php_rules.len(),
        25,
        "Expected exactly 25 PHP CWE rules in catalog, found {}",
        php_rules.len()
    );

    // Verify all 25 rules are CRITICAL or HIGH
    for r in &php_rules {
        assert!(
            r.severity == "CRITICAL" || r.severity == "HIGH",
            "Rule {} must have severity CRITICAL or HIGH, got {}",
            r.rule_id,
            r.severity
        );
        // Verify file detection based on CWE type
        assert!(
            SecurityLinter::is_file_eligible_for_cwe(&r.rule_id, "src/Controller/UserController.php"),
            "Rule {} should match .php file",
            r.rule_id
        );
        assert!(
            SecurityLinter::is_file_eligible_for_cwe(&r.rule_id, "views/index.phtml"),
            "Rule {} should match .phtml file",
            r.rule_id
        );
        assert!(
            !SecurityLinter::is_file_eligible_for_cwe(&r.rule_id, "src/main.rs"),
            "Rule {} should NOT match .rs file",
            r.rule_id
        );
        assert!(
            !SecurityLinter::is_file_eligible_for_cwe(&r.rule_id, "server.py"),
            "Rule {} should NOT match .py file",
            r.rule_id
        );
        assert!(
            r.matches_file("api/login.php"),
            "CveRuleDef::matches_file should match PHP file"
        );
    }
}

#[test]
fn test_php_file_detection_filters() {
    assert!(PhpSecurityRules::is_php_file("index.php"));
    assert!(PhpSecurityRules::is_php_file("views/template.phtml"));
    assert!(PhpSecurityRules::is_php_file("legacy/script.php3"));
    assert!(PhpSecurityRules::is_php_file("legacy/script.php7"));
    assert!(!PhpSecurityRules::is_php_file("main.rs"));
    assert!(!PhpSecurityRules::is_php_file("app.py"));
    assert!(!PhpSecurityRules::is_php_file("index.js"));
    assert!(!PhpSecurityRules::is_php_file("Contract.sol"));
}

#[test]
fn test_all_25_php_cwe_vulnerabilities_and_safe_remediations() {
    // 1. CWE-89: SQL Injection
    let vuln_sql = r#"<?php
$id = $_GET['id'];
$res = $pdo->query("SELECT * FROM users WHERE id = " . $id);
"#;
    let safe_sql = r#"<?php
$stmt = $pdo->prepare("SELECT * FROM users WHERE id = :id");
$stmt->execute(['id' => $id]);
"#;
    let v_sql = SecurityLinter::audit_file("src/UserRepo.php", vuln_sql, None);
    assert!(v_sql.iter().any(|v| v.rule_id == "CVE-PHP-SQL-INJECTION" && v.severity == "CRITICAL"));
    let s_sql = SecurityLinter::audit_file("src/UserRepo.php", safe_sql, None);
    assert!(!s_sql.iter().any(|v| v.rule_id == "CVE-PHP-SQL-INJECTION"));

    // 2. CWE-78: OS Command Injection
    let vuln_cmd = r#"<?php
$host = $_GET['host'];
system("ping -c 1 " . $host);
"#;
    let safe_cmd = r#"<?php
$host = $_GET['host'];
system("ping -c 1 " . escapeshellarg($host));
"#;
    let v_cmd = SecurityLinter::audit_file("src/Network.php", vuln_cmd, None);
    assert!(v_cmd.iter().any(|v| v.rule_id == "CVE-PHP-COMMAND-INJECTION" && v.severity == "CRITICAL"));
    let s_cmd = SecurityLinter::audit_file("src/Network.php", safe_cmd, None);
    assert!(!s_cmd.iter().any(|v| v.rule_id == "CVE-PHP-COMMAND-INJECTION"));

    // 3. CWE-94: Eval Code Injection
    let vuln_eval = r#"<?php
$code = $_POST['formula'];
eval($code);
"#;
    let safe_eval = r#"<?php
$val = intval($_POST['formula']);
"#;
    let v_eval = SecurityLinter::audit_file("src/Calc.php", vuln_eval, None);
    assert!(v_eval.iter().any(|v| v.rule_id == "CVE-PHP-EVAL-CODE-INJECTION" && v.severity == "CRITICAL"));
    let s_eval = SecurityLinter::audit_file("src/Calc.php", safe_eval, None);
    assert!(!s_eval.iter().any(|v| v.rule_id == "CVE-PHP-EVAL-CODE-INJECTION"));

    // 4. CWE-502: Unserialize Object Injection
    let vuln_deser = r#"<?php
$data = unserialize($_COOKIE['profile']);
"#;
    let safe_deser = r#"<?php
$data = unserialize($_COOKIE['profile'], ['allowed_classes' => false]);
"#;
    let v_deser = SecurityLinter::audit_file("src/Session.php", vuln_deser, None);
    assert!(v_deser.iter().any(|v| v.rule_id == "CVE-PHP-UNSERIALIZE-RCE" && v.severity == "CRITICAL"));
    let s_deser = SecurityLinter::audit_file("src/Session.php", safe_deser, None);
    assert!(!s_deser.iter().any(|v| v.rule_id == "CVE-PHP-UNSERIALIZE-RCE"));

    // 5. CWE-98: File Inclusion (RFI / LFI)
    let vuln_inc = r#"<?php
$page = $_GET['page'];
include "views/" . $page;
"#;
    let safe_inc = r#"<?php
$page = $_GET['page'];
if (in_array($page, ALLOWED_PAGES, true)) {
    include "views/" . $page;
}
"#;
    let v_inc = SecurityLinter::audit_file("src/Router.php", vuln_inc, None);
    assert!(v_inc.iter().any(|v| v.rule_id == "CVE-PHP-FILE-INCLUSION-RFI-LFI" && v.severity == "CRITICAL"));
    let s_inc = SecurityLinter::audit_file("src/Router.php", safe_inc, None);
    assert!(!s_inc.iter().any(|v| v.rule_id == "CVE-PHP-FILE-INCLUSION-RFI-LFI"));

    // 6. CWE-22: Path Traversal
    let vuln_trav = r#"<?php
$file = $_GET['filename'];
file_get_contents("uploads/" . $file);
"#;
    let safe_trav = r#"<?php
$file = basename($_GET['filename']);
file_get_contents("uploads/" . $file);
"#;
    let v_trav = SecurityLinter::audit_file("src/FileViewer.php", vuln_trav, None);
    assert!(v_trav.iter().any(|v| v.rule_id == "CVE-PHP-PATH-TRAVERSAL" && v.severity == "HIGH"));
    let s_trav = SecurityLinter::audit_file("src/FileViewer.php", safe_trav, None);
    assert!(!s_trav.iter().any(|v| v.rule_id == "CVE-PHP-PATH-TRAVERSAL"));

    // 7. CWE-79: XSS Output Unescaped
    let vuln_xss = r#"<?php
echo $_GET['search'];
"#;
    let safe_xss = r#"<?php
echo htmlspecialchars($_GET['search'], ENT_QUOTES, 'UTF-8');
"#;
    let v_xss = SecurityLinter::audit_file("views/search.phtml", vuln_xss, None);
    assert!(v_xss.iter().any(|v| v.rule_id == "CVE-PHP-XSS-UNESCAPED-OUTPUT" && v.severity == "HIGH"));
    let s_xss = SecurityLinter::audit_file("views/search.phtml", safe_xss, None);
    assert!(!s_xss.iter().any(|v| v.rule_id == "CVE-PHP-XSS-UNESCAPED-OUTPUT"));

    // 8. CWE-918: SSRF
    let vuln_ssrf = r#"<?php
$target = $_GET['webhook'];
curl_setopt($ch, CURLOPT_URL, $target);
"#;
    let safe_ssrf = r#"<?php
// Validate with FILTER_FLAG_NO_PRIV_RANGE
if (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE)) {
    curl_setopt($ch, CURLOPT_URL, $target);
}
"#;
    let v_ssrf = SecurityLinter::audit_file("src/WebhookClient.php", vuln_ssrf, None);
    assert!(v_ssrf.iter().any(|v| v.rule_id == "CVE-PHP-SSRF-CURL-STREAM" && v.severity == "HIGH"));
    let s_ssrf = SecurityLinter::audit_file("src/WebhookClient.php", safe_ssrf, None);
    assert!(!s_ssrf.iter().any(|v| v.rule_id == "CVE-PHP-SSRF-CURL-STREAM"));

    // 9. CWE-434: Unrestricted File Upload
    let vuln_upload = r#"<?php
move_uploaded_file($_FILES['avatar']['tmp_name'], "uploads/" . $_FILES['avatar']['name']);
"#;
    let safe_upload = r#"<?php
$newName = bin2hex(random_bytes(16)) . '.png';
move_uploaded_file($_FILES['avatar']['tmp_name'], "uploads/" . $newName);
"#;
    let v_upload = SecurityLinter::audit_file("src/UploadHandler.php", vuln_upload, None);
    assert!(v_upload.iter().any(|v| v.rule_id == "CVE-PHP-UNRESTRICTED-FILE-UPLOAD" && v.severity == "CRITICAL"));
    let s_upload = SecurityLinter::audit_file("src/UploadHandler.php", safe_upload, None);
    assert!(!s_upload.iter().any(|v| v.rule_id == "CVE-PHP-UNRESTRICTED-FILE-UPLOAD"));

    // 10. CWE-611: XXE Injection
    let vuln_xxe = r#"<?php
$xml = simplexml_load_string($raw_xml, 'SimpleXMLElement', LIBXML_NOENT);
"#;
    let safe_xxe = r#"<?php
libxml_disable_entity_loader(true);
$xml = simplexml_load_string($raw_xml, 'SimpleXMLElement', LIBXML_NONET);
"#;
    let v_xxe = SecurityLinter::audit_file("src/XmlParser.php", vuln_xxe, None);
    assert!(v_xxe.iter().any(|v| v.rule_id == "CVE-PHP-XXE-LIBXML-INJECTION" && v.severity == "HIGH"));
    let s_xxe = SecurityLinter::audit_file("src/XmlParser.php", safe_xxe, None);
    assert!(!s_xxe.iter().any(|v| v.rule_id == "CVE-PHP-XXE-LIBXML-INJECTION"));

    // 11. CWE-384 / CWE-287: Session Fixation
    let vuln_session = r#"<?php
$_SESSION['user_id'] = $user['id'];
"#;
    let safe_session = r#"<?php
session_regenerate_id(true);
$_SESSION['user_id'] = $user['id'];
"#;
    let v_sess = SecurityLinter::audit_file("src/Auth/Login.php", vuln_session, None);
    assert!(v_sess.iter().any(|v| v.rule_id == "CVE-PHP-SESSION-FIXATION-NO-REGEN" && v.severity == "HIGH"));
    let s_sess = SecurityLinter::audit_file("src/Auth/Login.php", safe_session, None);
    assert!(!s_sess.iter().any(|v| v.rule_id == "CVE-PHP-SESSION-FIXATION-NO-REGEN"));

    // 12. CWE-312 / CWE-319: Cookie Sensitive Plaintext
    let vuln_cookie = r#"<?php
setcookie("auth_token", $token, time() + 3600, "/", "", false, false);
"#;
    let safe_cookie = r#"<?php
setcookie("auth_token", $token, ['expires' => time() + 3600, 'path' => '/', 'secure' => true, 'httponly' => true, 'samesite' => 'Strict']);
"#;
    let v_cookie = SecurityLinter::audit_file("src/CookieManager.php", vuln_cookie, None);
    assert!(v_cookie.iter().any(|v| v.rule_id == "CVE-PHP-COOKIE-SENSITIVE-PLAINTEXT" && v.severity == "HIGH"));
    let s_cookie = SecurityLinter::audit_file("src/CookieManager.php", safe_cookie, None);
    assert!(!s_cookie.iter().any(|v| v.rule_id == "CVE-PHP-COOKIE-SENSITIVE-PLAINTEXT"));

    // 13. CWE-362: File Race Condition TOCTOU
    let vuln_toctou = r#"<?php
if (!file_exists($lock)) {
    file_put_contents($lock, "1");
}
"#;
    let safe_toctou = r#"<?php
$fp = fopen($lock, "c+");
if (flock($fp, LOCK_EX)) {
    file_put_contents($lock, "1");
}
"#;
    let v_toctou = SecurityLinter::audit_file("src/LockService.php", vuln_toctou, None);
    assert!(v_toctou.iter().any(|v| v.rule_id == "CVE-PHP-FILE-LOCKING-TOCTOU" && v.severity == "HIGH"));
    let s_toctou = SecurityLinter::audit_file("src/LockService.php", safe_toctou, None);
    assert!(!s_toctou.iter().any(|v| v.rule_id == "CVE-PHP-FILE-LOCKING-TOCTOU"));

    // 14. CWE-327: Broken Crypto MD5/SHA1
    let vuln_crypto = r#"<?php
$pass_hash = md5($_POST['password']);
"#;
    let safe_crypto = r#"<?php
$pass_hash = password_hash($_POST['password'], PASSWORD_ARGON2ID);
"#;
    let v_crypto = SecurityLinter::audit_file("src/PasswordService.php", vuln_crypto, None);
    assert!(v_crypto.iter().any(|v| v.rule_id == "CVE-PHP-BROKEN-CRYPTO-MD5-SHA1" && v.severity == "HIGH"));
    let s_crypto = SecurityLinter::audit_file("src/PasswordService.php", safe_crypto, None);
    assert!(!s_crypto.iter().any(|v| v.rule_id == "CVE-PHP-BROKEN-CRYPTO-MD5-SHA1"));

    // 15. CWE-113: Header Injection CRLF
    let vuln_header = r#"<?php
header("Location: " . $_GET['redirect']);
"#;
    let safe_header = r#"<?php
$clean = str_replace(["\r", "\n"], '', $_GET['redirect']);
header("Location: " . $clean);
"#;
    let v_header = SecurityLinter::audit_file("src/Redirector.php", vuln_header, None);
    assert!(v_header.iter().any(|v| v.rule_id == "CVE-PHP-HEADER-INJECTION-CRLF" && v.severity == "HIGH"));
    let s_header = SecurityLinter::audit_file("src/Redirector.php", safe_header, None);
    assert!(!s_header.iter().any(|v| v.rule_id == "CVE-PHP-HEADER-INJECTION-CRLF"));

    // 16. CWE-621: Variable Overwrite Extract
    let vuln_extract = r#"<?php
extract($_POST);
"#;
    let safe_extract = r#"<?php
extract($_POST, EXTR_SKIP);
"#;
    let v_extract = SecurityLinter::audit_file("src/Config.php", vuln_extract, None);
    assert!(v_extract.iter().any(|v| v.rule_id == "CVE-PHP-VARIABLE-OVERWRITE-EXTRACT" && v.severity == "CRITICAL"));
    let s_extract = SecurityLinter::audit_file("src/Config.php", safe_extract, None);
    assert!(!s_extract.iter().any(|v| v.rule_id == "CVE-PHP-VARIABLE-OVERWRITE-EXTRACT"));

    // 17. CWE-88: Mail Parameter Injection
    let vuln_mail = r#"<?php
mail($to, $subject, $message, $headers, "-f" . $sender);
"#;
    let safe_mail = r#"<?php
mail($to, $subject, $message, $headers, "-f" . escapeshellcmd($sender));
"#;
    let v_mail = SecurityLinter::audit_file("src/Mailer.php", vuln_mail, None);
    assert!(v_mail.iter().any(|v| v.rule_id == "CVE-PHP-MAIL-PARAM-INJECTION" && v.severity == "CRITICAL"));
    let s_mail = SecurityLinter::audit_file("src/Mailer.php", safe_mail, None);
    assert!(!s_mail.iter().any(|v| v.rule_id == "CVE-PHP-MAIL-PARAM-INJECTION"));

    // 18. CWE-1336: Server-Side Template Injection (SSTI)
    let vuln_ssti = r#"<?php
$twig->createTemplate($_GET['template'])->render();
"#;
    let safe_ssti = r#"<?php
$twig->render("profile.html.twig", ['template' => "safe"]);
"#;
    let v_ssti = SecurityLinter::audit_file("src/TemplateEngine.php", vuln_ssti, None);
    assert!(v_ssti.iter().any(|v| v.rule_id == "CVE-PHP-TEMPLATE-INJECTION-SSTI" && v.severity == "CRITICAL"));
    let s_ssti = SecurityLinter::audit_file("src/TemplateEngine.php", safe_ssti, None);
    assert!(!s_ssti.iter().any(|v| v.rule_id == "CVE-PHP-TEMPLATE-INJECTION-SSTI"));

    // 19. CWE-295: Disabled SSL Verification
    let vuln_ssl = r#"<?php
curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false);
"#;
    let safe_ssl = r#"<?php
curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, true);
"#;
    let v_ssl = SecurityLinter::audit_file("src/PaymentGateway.php", vuln_ssl, None);
    assert!(v_ssl.iter().any(|v| v.rule_id == "CVE-PHP-DISABLED-SSL-VERIFY" && v.severity == "HIGH"));
    let s_ssl = SecurityLinter::audit_file("src/PaymentGateway.php", safe_ssl, None);
    assert!(!s_ssl.iter().any(|v| v.rule_id == "CVE-PHP-DISABLED-SSL-VERIFY"));

    // 20. CWE-697: Loose Comparison Type Juggling
    let vuln_juggle = r#"<?php
if ($stored_token == $_GET['token']) {
    login_user();
}
"#;
    let safe_juggle = r#"<?php
if (hash_equals($stored_token, $_GET['token'])) {
    login_user();
}
"#;
    let v_juggle = SecurityLinter::audit_file("src/AuthService.php", vuln_juggle, None);
    assert!(v_juggle.iter().any(|v| v.rule_id == "CVE-PHP-TYPE-JUGGLING-AUTH" && v.severity == "HIGH"));
    let s_juggle = SecurityLinter::audit_file("src/AuthService.php", safe_juggle, None);
    assert!(!s_juggle.iter().any(|v| v.rule_id == "CVE-PHP-TYPE-JUGGLING-AUTH"));

    // 21. CWE-338: Cryptographically Weak PRNG for Secrets
    let vuln_prng = r#"<?php
$reset_token = md5(uniqid(rand(), true));
"#;
    let safe_prng = r#"<?php
$reset_token = bin2hex(random_bytes(32));
"#;
    let v_prng = SecurityLinter::audit_file("src/PasswordReset.php", vuln_prng, None);
    assert!(v_prng.iter().any(|v| v.rule_id == "CVE-PHP-WEAK-PRNG-SECRETS" && v.severity == "HIGH"));
    let s_prng = SecurityLinter::audit_file("src/PasswordReset.php", safe_prng, None);
    assert!(!s_prng.iter().any(|v| v.rule_id == "CVE-PHP-WEAK-PRNG-SECRETS"));

    // 22. CWE-90: LDAP Injection
    let vuln_ldap = r#"<?php
$res = ldap_search($ds, "dc=example,dc=com", "uid=" . $_POST['username']);
"#;
    let safe_ldap = r#"<?php
$clean_user = ldap_escape($_POST['username'], null, LDAP_ESCAPE_FILTER);
$res = ldap_search($ds, "dc=example,dc=com", "uid=" . $clean_user);
"#;
    let v_ldap = SecurityLinter::audit_file("src/LdapAuth.php", vuln_ldap, None);
    assert!(v_ldap.iter().any(|v| v.rule_id == "CVE-PHP-LDAP-INJECTION" && v.severity == "HIGH"));
    let s_ldap = SecurityLinter::audit_file("src/LdapAuth.php", safe_ldap, None);
    assert!(!s_ldap.iter().any(|v| v.rule_id == "CVE-PHP-LDAP-INJECTION"));

    // 23. CWE-943: NoSQL Injection (MongoDB)
    let vuln_nosql = r#"<?php
$user = $db->users->findOne(['username' => $_POST['username'], 'password' => $_POST['password']]);
"#;
    let safe_nosql = r#"<?php
$user = $db->users->findOne(['username' => (string)$_POST['username'], 'password' => (string)$_POST['password']]);
"#;
    let v_nosql = SecurityLinter::audit_file("src/MongoRepo.php", vuln_nosql, None);
    assert!(v_nosql.iter().any(|v| v.rule_id == "CVE-PHP-NOSQL-INJECTION" && v.severity == "HIGH"));
    let s_nosql = SecurityLinter::audit_file("src/MongoRepo.php", safe_nosql, None);
    assert!(!s_nosql.iter().any(|v| v.rule_id == "CVE-PHP-NOSQL-INJECTION"));

    // 24. CWE-770 / CWE-22: Zip Slip
    let vuln_zip = r#"<?php
$zip->extractTo("/var/www/uploads/");
"#;
    let safe_zip = r#"<?php
if (str_starts_with(realpath($dest), realpath($target_dir))) {
    $zip->extractTo($dest);
}
"#;
    let v_zip = SecurityLinter::audit_file("src/ArchiveExtractor.php", vuln_zip, None);
    assert!(v_zip.iter().any(|v| v.rule_id == "CVE-PHP-ZIP-SLIP-EXTRACTION" && v.severity == "HIGH"));
    let s_zip = SecurityLinter::audit_file("src/ArchiveExtractor.php", safe_zip, None);
    assert!(!s_zip.iter().any(|v| v.rule_id == "CVE-PHP-ZIP-SLIP-EXTRACTION"));

    // 25. CWE-209 / CWE-215: Exposed Debug Config
    let vuln_debug = r#"<?php
ini_set('display_errors', '1');
"#;
    let safe_debug = r#"<?php
ini_set('display_errors', '0');
"#;
    let v_debug = SecurityLinter::audit_file("src/Bootstrap.php", vuln_debug, None);
    assert!(v_debug.iter().any(|v| v.rule_id == "CVE-PHP-EXPOSED-DEBUG-CONFIG" && v.severity == "HIGH"));
    let s_debug = SecurityLinter::audit_file("src/Bootstrap.php", safe_debug, None);
    assert!(!s_debug.iter().any(|v| v.rule_id == "CVE-PHP-EXPOSED-DEBUG-CONFIG"));
}

#[test]
fn test_php_rules_do_not_trigger_on_rust_or_python_files() {
    let fake_php_code_in_rust = r#"
// This Rust file contains substrings that look like PHP:
// system("ping -c 1 " . host);
// eval(code);
pub fn test_mock() {
    let query = "SELECT * FROM users WHERE id = ";
}
"#;
    let violations = SecurityLinter::audit_file("src/lib.rs", fake_php_code_in_rust, None);
    assert!(
        !violations.iter().any(|v| v.rule_id.starts_with("CVE-PHP-")),
        "PHP rules must NEVER trigger on Rust files"
    );
}
