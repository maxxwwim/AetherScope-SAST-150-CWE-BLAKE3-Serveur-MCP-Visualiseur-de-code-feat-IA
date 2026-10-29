use crate::linter::{CveRuleDef, LinterViolation};

pub struct AdvancedDefiRules;

impl AdvancedDefiRules {
    pub fn definitions() -> Vec<CveRuleDef> {
        vec![
            CveRuleDef {
                rule_id: "DEFI-ASYMMETRIC-ROUNDING".to_string(),
                cve_id: "CWE-682 / ASYMMETRIC_ROUNDING_RISK".to_string(),
                name: "Arrondi Asymétrique en Faveur de l'Attaquant (Vaults / Taxe Économique)".to_string(),
                category: "DeFi / Invariants".to_string(),
                severity: "HIGH".to_string(),
                description: "Division entière brute ou calcul de ratio dans les conversions d'actifs/shares ou taxes économiques sans fonction d'arrondi sécurisée (mulDivUp/mulDivDown ou .max(1)), permettant le vol de micro-parts ou l'évasion de frais.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "DEFI-STALE-CACHE-AFTER-CALL".to_string(),
                cve_id: "CWE-362 / STALE_CACHE_TOCTOU".to_string(),
                name: "Lecture d'État / Cache Périmé Après Appel Externe (VaultCache TOCTOU)".to_string(),
                category: "DeFi / Invariants".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Réutilisation d'un snapshot ou cache d'état local (VaultCache, local_balance, snapshot) après un appel externe (.call, HookTarget, .await) sans réactualisation préalable, entraînant des décisions sur un solde corrompu.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "EVC-HOOK-BYPASS".to_string(),
                cve_id: "CWE-841 / EVC_HOOK_BYPASS_RISK".to_string(),
                name: "Contournement de Collatéral / Batch Reentrancy via Hook EVC Non Sécurisé".to_string(),
                category: "DeFi / Invariants".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de mutations d'état ou transferts dans un hook/callback sans vérification de l'état global du connecteur EVC ou des contrôles de liquidité différés (deferLiquidityCheck / nonReentrant).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER".to_string(),
                cve_id: "LOGIC-CRITICAL / ORACLE_MIN_MAX_CIRCUIT_BREAKER".to_string(),
                name: "Absence de Contrôle des Bornes Min/Max d'Oracle (Effondrement de Prix)".to_string(),
                category: "DeFi / Oracles".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Interrogation d'un flux d'oracle sans vérification que le prix retourné ne percute pas les bornes matérielles minAnswer ou maxAnswer de l'agrégateur (ou absence de circuit breaker / oracle de secours), permettant de valoriser un token effondré à son plancher artificiel pour siphonner les vaults.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "DEFI-ORACLE-L2-SEQUENCER-RACE".to_string(),
                cve_id: "LOGIC-CRITICAL / L2_SEQUENCER_ORACLE_RACE".to_string(),
                name: "Course d'Oracle sur Séquenceur L2 Sans Période de Grâce (Arbitrum / Base / Optimism)".to_string(),
                category: "DeFi / Oracles".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Lecture de prix Chainlink sur Layer 2 sans interroger le sequencerUptimeFeed et sans imposer de délai de grâce (GRACE_PERIOD_TIME) après le redémarrage d'un séquenceur en panne, exposant les utilisateurs à des liquidations massives à cours périmé.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "DEFI-TWAP-SHORT-WINDOW".to_string(),
                cve_id: "LOGIC-CRITICAL / TWAP_SHORT_WINDOW".to_string(),
                name: "Fenêtre TWAP AMM Inférieure à 30 Minutes Manipulable par MEV Multi-Blocs".to_string(),
                category: "DeFi / Oracles".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Fenêtre temporelle d'oracle TWAP Uniswap v3 ou AMM inférieure à 1800 secondes (30 minutes) ou non plafonnée par un minimum strict, permettant une manipulation rentable de la moyenne géométrique des prix via des pots-de-vin MEV multi-blocs.".to_string(),
                default_enabled: true,
            },
        ]
    }

    /// Audit syntaxique & taint tracking pour les 3 règles avancées DeFi & EVC
    pub fn audit_file(
        file_path: &str,
        content: &str,
        is_test_file: bool,
        is_interface_or_types: bool,
    ) -> Vec<LinterViolation> {
        let mut violations = Vec::new();
        if is_test_file || is_interface_or_types {
            return violations;
        }

        let lines: Vec<&str> = content.lines().collect();
        let lower_path = file_path.to_lowercase();
        let is_sol = lower_path.ends_with(".sol");
        let is_rust = lower_path.ends_with(".rs");

        // ---------------------------------------------------------------------
        // 1. DEFI-ASYMMETRIC-ROUNDING (CWE-682)
        // ---------------------------------------------------------------------
        for (idx, &line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            // Solidity conversions (Vault / ERC-4626 / Synthetics)
            if is_sol && (lower_path.contains("vault") || lower_path.contains("token") || lower_path.contains("synth") || lower_path.contains("pool")) {
                let is_conversion_scope = lines.iter().take(idx + 1).rev().take(15).any(|l| {
                    l.contains("convertToShares")
                        || l.contains("convertToAssets")
                        || l.contains("previewDeposit")
                        || l.contains("previewWithdraw")
                        || l.contains("previewRedeem")
                        || l.contains("previewMint")
                });

                if is_conversion_scope
                    && (trimmed.contains(" / ") || (trimmed.contains('/') && !trimmed.contains("//")))
                    && !trimmed.contains("super.")
                    && !trimmed.ends_with(';')
                    && !content.contains("mulDiv")
                    && !content.contains("mulDivUp")
                    && !content.contains("mulDivDown")
                    && !content.contains("Rounding.Floor")
                    && !content.contains("Rounding.Ceil")
                    && !content.contains("FixedPointMathLib")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "DEFI-ASYMMETRIC-ROUNDING".to_string(),
                        cve_id: "CWE-682 / ASYMMETRIC_ROUNDING_RISK".to_string(),
                        severity: "HIGH".to_string(),
                        message: "☣️ [CWE-682] ASYMMETRIC_ROUNDING_RISK : Conversion calculate ratio with integer division without explicit directional rounding (mulDivUp for protocol / mulDivDown for user): allows micro-minting inflation exploitation".to_string(),
                        transmitted_data: Some("conversion_ratio_rounding".to_string()),
                        transmitted_type: Some("uint256 (Asymmetric Integer Division)".to_string()),
                        remediation: "Use OpenZeppelin Math.mulDiv(assets, shares, total, Math.Rounding.Floor) or mulDivUp to protect protocol solvency.".to_string(),
                    });
                }
            }

            // Rust Economy / Taxes (Habbo / MMO / Tokenomics)
            if is_rust && (lower_path.contains("economy") || lower_path.contains("tax") || lower_path.contains("market") || lower_path.contains("wallet")) {
                let is_fee_calc = trimmed.contains("tax =")
                    || trimmed.contains("fee =")
                    || trimmed.contains("tax_amount")
                    || trimmed.contains("fee_amount")
                    || (trimmed.contains("shares") && trimmed.contains('/'));

                if is_fee_calc
                    && trimmed.contains('/')
                    && !trimmed.contains(".max(1")
                    && !trimmed.contains(".saturating_")
                    && !trimmed.contains("ceil")
                    && !content.contains("mul_div")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "DEFI-ASYMMETRIC-ROUNDING".to_string(),
                        cve_id: "CWE-682 / ASYMMETRIC_ROUNDING_RISK".to_string(),
                        severity: "HIGH".to_string(),
                        message: "☣️ [CWE-682] ASYMMETRIC_ROUNDING_RISK : Fee or token ratio calculated with truncating division without minimum rounding (.max(1)): micro-transactions evade taxes completely".to_string(),
                        transmitted_data: Some("tax_truncated_fee".to_string()),
                        transmitted_type: Some("u64 / u128 (Truncated Tax Integer Division)".to_string()),
                        remediation: "Enforce safe ceil rounding or floor with .max(1): fee.checked_div(divisor).map(|v| v.max(1)).".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 2. DEFI-STALE-CACHE-AFTER-CALL (CWE-362 / TOCTOU Cache)
        // ---------------------------------------------------------------------
        // Taint tracking : identifier les fonctions qui chargent un snapshot/cache, font un appel externe, puis réutilisent le cache sans le recharger
        let mut in_function = false;
        let mut fn_has_cache_load = false;
        let mut cache_var_name = String::new();
        let mut external_call_line: Option<usize> = None;

        for (idx, &line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("function ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                in_function = true;
                fn_has_cache_load = false;
                cache_var_name.clear();
                external_call_line = None;
            }

            if in_function {
                // Détection de snapshot ou cache local
                if trimmed.contains("VaultCache ") || trimmed.contains("local_balance") || trimmed.contains("let mut cache") || trimmed.contains("let mut snapshot") || trimmed.contains("Snapshot memory ") {
                    fn_has_cache_load = true;
                    if trimmed.contains("vaultCache") {
                        cache_var_name = "vaultCache".to_string();
                    } else if trimmed.contains("snapshot") {
                        cache_var_name = "snapshot".to_string();
                    } else if trimmed.contains("local_balance") {
                        cache_var_name = "local_balance".to_string();
                    } else {
                        cache_var_name = "cache".to_string();
                    }
                }

                // Détection d'appel externe ou de hook
                let is_ext_call = trimmed.contains(".call(")
                    || trimmed.contains(".call{")
                    || trimmed.contains(".invoke(")
                    || trimmed.contains("HookTarget")
                    || (is_rust && trimmed.contains(".await") && (trimmed.contains("rpc") || trimmed.contains("client") || trimmed.contains("http")));

                if fn_has_cache_load && is_ext_call && external_call_line.is_none() {
                    external_call_line = Some(line_no);
                }

                // Si appel externe a déjà eu lieu et qu'on utilise encore le cache sans update
                if let Some(call_l) = external_call_line {
                    let has_reuse_after_call = if line_no > call_l {
                        true
                    } else if line_no == call_l {
                        let ext_call_pos = trimmed.find(".call").or_else(|| trimmed.find(".invoke")).unwrap_or(0);
                        let cache_pos = trimmed.rfind(&cache_var_name).unwrap_or(0);
                        cache_pos > ext_call_pos
                    } else {
                        false
                    };

                    if has_reuse_after_call {
                        // Réutilisation du cache
                        let uses_cache = !cache_var_name.is_empty() && trimmed.contains(&cache_var_name);
                        let is_cache_reload = trimmed.contains("updateVaultCache")
                            || trimmed.contains("reload_cache")
                            || trimmed.contains("refresh(")
                            || trimmed.contains("load_cache")
                            || trimmed.contains("fetch_latest");

                        if is_cache_reload {
                            external_call_line = None; // Cache réactualisé, invariant restauré
                        } else if uses_cache
                            && (trimmed.contains(".balance") || trimmed.contains(".assets") || trimmed.contains(".liability") || trimmed.contains(".shares") || trimmed.contains("collateral"))
                            && !trimmed.contains("updateVaultCache")
                        {
                            violations.push(LinterViolation {
                                file_path: file_path.to_string(),
                                line_number: line_no,
                                rule_id: "DEFI-STALE-CACHE-AFTER-CALL".to_string(),
                                cve_id: "CWE-362 / STALE_CACHE_TOCTOU".to_string(),
                                severity: "CRITICAL".to_string(),
                                message: format!("☣️ [CWE-362] STALE_CACHE_TOCTOU : Cached state snapshot '{}' reused after external call at line {} without refresh: state desynchronization vulnerability", cache_var_name, call_l),
                                transmitted_data: Some(format!("stale_{}_snapshot", cache_var_name)),
                                transmitted_type: Some("CacheSnapshot (Outdated Memory Snapshot Across External Boundary)".to_string()),
                                remediation: "Reload or recompute the state cache immediately after external calls before calculating collateral or balance deltas.".to_string(),
                            });
                            external_call_line = None; // Une seule alerte par fonction
                        }
                    }
                }
            }

            if trimmed == "}" {
                in_function = false;
                fn_has_cache_load = false;
                cache_var_name.clear();
                external_call_line = None;
            }
        }

        // ---------------------------------------------------------------------
        // 3. EVC-HOOK-BYPASS (CWE-841 / Batch Reentrancy via Hook)
        // ---------------------------------------------------------------------
        for (idx, &line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();

            let is_hook_handler = trimmed.contains("function checkVaultStatus")
                || trimmed.contains("function onVaultCall")
                || trimmed.contains("function hookTarget")
                || trimmed.contains("function preOperationHook")
                || trimmed.contains("function postOperationHook");

            if is_hook_handler
                && !trimmed.ends_with(';')
                && !trimmed.contains("super.")
                && (content.contains(".transfer(") || content.contains(".transferFrom(") || content.contains(".call(") || content.contains("emit "))
                && !content.contains("nonReentrant")
                && !content.contains("isChecksDeferred")
                && !content.contains("areChecksInProgress")
                && !content.contains("onlyEVC")
                && !content.contains("onlyVault")
            {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "EVC-HOOK-BYPASS".to_string(),
                    cve_id: "CWE-841 / EVC_HOOK_BYPASS_RISK".to_string(),
                    severity: "CRITICAL".to_string(),
                    message: "☣️ [CWE-841] EVC_HOOK_BYPASS_RISK : Hook or callback allows state mutation without nonReentrant guard or checking EVC deferred liquidity checks: vulnerable to transient batch reentrancy bypass".to_string(),
                    transmitted_data: Some("evc_unverified_execution_context".to_string()),
                    transmitted_type: Some("Callback / Hook (Unchecked Transient Deferred Check State)".to_string()),
                    remediation: "Apply OpenZeppelin nonReentrant modifier and assert !evc.isChecksDeferred(account) || evc.areChecksInProgress().".to_string(),
                });
            }
        }

        // ---------------------------------------------------------------------
        // 4. DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER (Crash / Hardcoded Bound Clamping)
        // ---------------------------------------------------------------------
        if is_sol && (lower_path.contains("oracle") || lower_path.contains("adapter") || lower_path.contains("price") || lower_path.contains("feed")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.contains("latestRoundData(")
                    && !trimmed.contains("function latestRoundData")
                    && (content.contains("int256") || content.contains("int ") || content.contains("answer"))
                    && !content.contains("minAnswer")
                    && !content.contains("maxAnswer")
                    && !content.contains("minPrice")
                    && !content.contains("maxPrice")
                    && !content.contains("MIN_ANSWER")
                    && !content.contains("MAX_ANSWER")
                    && !content.contains("MIN_PRICE")
                    && !content.contains("MAX_PRICE")
                    && !content.contains("circuitBreaker")
                    && !content.contains("fallbackOracle")
                    && !content.contains("upperBound")
                    && !content.contains("lowerBound")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER".to_string(),
                        cve_id: "LOGIC-CRITICAL / ORACLE_MIN_MAX_CIRCUIT_BREAKER".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [LOGIC-CRITICAL] ORACLE_MIN_MAX_CIRCUIT_BREAKER : Chainlink latestRoundData() queried without minAnswer/maxAnswer bound check or fallback circuit breaker: extreme price drop will cap price artificially (oracle bounds attack vector)".to_string(),
                        transmitted_data: Some("oracle_answer_unbounded".to_string()),
                        transmitted_type: Some("int256 (Hardcoded Feed Bound Clamping)".to_string()),
                        remediation: "Query aggregator minAnswer / maxAnswer bounds and revert or trigger fallback oracle if answer <= minAnswer || answer >= maxAnswer.".to_string(),
                    });
                    break;
                }
            }
        }

        // ---------------------------------------------------------------------
        // 5. DEFI-ORACLE-L2-SEQUENCER-RACE (Arbitrum / Base / Optimism Downtime)
        // ---------------------------------------------------------------------
        let has_l2_context = lower_path.contains("arbitrum")
            || lower_path.contains("optimism")
            || lower_path.contains("base")
            || lower_path.contains("l2")
            || content.to_lowercase().contains("arbitrum")
            || content.to_lowercase().contains("optimism")
            || content.to_lowercase().contains("base")
            || content.to_lowercase().contains("sequencer");

        if is_sol && has_l2_context {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.contains("latestRoundData(")
                    && !trimmed.contains("function latestRoundData")
                    && !content.contains("sequencerUptimeFeed")
                    && !content.contains("isSequencerActive")
                    && !content.contains("gracePeriod")
                    && !content.contains("GRACE_PERIOD")
                    && !content.contains("startedAt +")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "DEFI-ORACLE-L2-SEQUENCER-RACE".to_string(),
                        cve_id: "LOGIC-CRITICAL / L2_SEQUENCER_ORACLE_RACE".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [LOGIC-CRITICAL] L2_SEQUENCER_ORACLE_RACE : L2 oracle does not check Sequencer Uptime Feed or lacks restart grace period (GRACE_PERIOD_TIME): vulnerable to stale-price front-running and unfair liquidations during sequencer downtime".to_string(),
                        transmitted_data: Some("l2_sequencer_status".to_string()),
                        transmitted_type: Some("UptimeFeed (Missing Post-Restart Grace Period)".to_string()),
                        remediation: "Query AggregatorV2V3Interface(sequencerUptimeFeed).latestRoundData() and enforce require(answer == 0 && block.timestamp - startedAt > GRACE_PERIOD_TIME, 'Sequencer restarted').".to_string(),
                    });
                    break;
                }
            }
        }

        // ---------------------------------------------------------------------
        // 6. DEFI-TWAP-SHORT-WINDOW (AMM Multi-Block MEV Manipulation)
        // ---------------------------------------------------------------------
        if is_sol && (content.contains(".observe(") || content.contains("OracleLibrary.consult") || content.contains("observeSingle") || lower_path.contains("uniswap") || lower_path.contains("twap")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                let has_short_window = (trimmed.contains("secondsAgo") || trimmed.contains("twapWindow") || trimmed.contains("twapPeriod") || trimmed.contains("period"))
                    && (trimmed.contains(" = 60;")
                        || trimmed.contains(" = 120;")
                        || trimmed.contains(" = 300;")
                        || trimmed.contains(" = 600;")
                        || trimmed.contains(" = 900;")
                        || trimmed.contains(" = 1 minutes;")
                        || trimmed.contains(" = 5 minutes;")
                        || trimmed.contains(" = 10 minutes;")
                        || trimmed.contains(" = 15 minutes;")
                        || (trimmed.contains(" < 1800") && trimmed.contains("revert"))
                        || (trimmed.contains(" < 30 minutes") && trimmed.contains("revert")));

                let is_unbounded_setter = (trimmed.contains("function setTwap") || trimmed.contains("function setPeriod") || trimmed.contains("constructor("))
                    && (content.contains("secondsAgo") || content.contains("twapPeriod"))
                    && !content.contains("1800")
                    && !content.contains("30 minutes")
                    && !content.contains("MIN_TWAP")
                    && !content.contains("MIN_PERIOD");

                if has_short_window || (is_unbounded_setter && trimmed.contains("function set")) {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "DEFI-TWAP-SHORT-WINDOW".to_string(),
                        cve_id: "LOGIC-CRITICAL / TWAP_SHORT_WINDOW".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [LOGIC-CRITICAL] TWAP_SHORT_WINDOW : AMM TWAP window is shorter than 1800s (30 minutes) or lacks strict minimum floor: vulnerable to multi-block MEV geometric mean manipulation".to_string(),
                        transmitted_data: Some("amm_twap_seconds_ago".to_string()),
                        transmitted_type: Some("uint32 (Manipulable Short TWAP Window)".to_string()),
                        remediation: "Enforce twapWindow >= 1800 (at least 30 minutes) to make multi-block MEV pool manipulation economically unfeasible.".to_string(),
                    });
                    break;
                }
            }
        }

        violations
    }
}
