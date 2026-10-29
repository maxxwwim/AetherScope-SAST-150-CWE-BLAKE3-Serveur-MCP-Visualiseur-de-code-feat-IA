use crate::linter::{CveRuleDef, LinterViolation};

/// Règles de sécurité d'élite / ultra-spécialisées (ZK-SNARKs, Solana/Anchor, Rust UB & Async, DeFi Read-Only, Rollup L2, ERC-4337, CosmWasm).
pub struct DeepSecurityRules;

impl DeepSecurityRules {
    pub fn definitions() -> Vec<CveRuleDef> {
        vec![
            CveRuleDef {
                rule_id: "CVE-ZK-WEAK-FIAT-SHAMIR".to_string(),
                cve_id: "CWE-347 / WEAK-FIAT-SHAMIR-TRANSCRIPT-BYPASS".to_string(),
                name: "ZK-SNARKs: Weak Fiat-Shamir Transformation (Missing Public Input / Instance Binding)".to_string(),
                category: "Cryptography / Zero-Knowledge".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Challenge sampling in non-interactive ZK proof without absorbing public inputs or instance statements into the Fiat-Shamir transcript: allows forgery of valid proofs for arbitrary statements (Frozen Heart vulnerability).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-ZK-UNDERCONSTRAINED-SIGNAL".to_string(),
                cve_id: "CWE-682 / ZK-UNDERCONSTRAINED-CIRCUIT".to_string(),
                name: "ZK Circuits: Underconstrained Witness Assignment without R1CS Enforcement".to_string(),
                category: "Cryptography / Zero-Knowledge".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Witness assignment operator '<--' used in Circom or unconstrained variable in Halo2 circuit without corresponding equality constraint '===' or boolean quadratic constraint: allows arbitrary witness poisoning.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-DEFI-READ-ONLY-REENTRANCY".to_string(),
                cve_id: "CWE-841 / DEFI-READ-ONLY-REENTRANCY".to_string(),
                name: "DeFi: Read-Only Reentrancy on Curve / Balancer AMM Virtual Price".to_string(),
                category: "DeFi / Oracles".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Invocation of view price feeds (get_virtual_price(), getRate(), getPoolTokens()) during intermediate state without reentrancy guard or lock verification: allows flash loan price manipulation & unfair liquidation.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-SOLANA-ACCOUNT-CONFUSION-DISCRIMINATOR".to_string(),
                cve_id: "CWE-843 / SOLANA-ACCOUNT-CONFUSION-DISCRIMINATOR".to_string(),
                name: "Solana / Anchor: Missing 8-Byte Discriminator Validation (Account Cosplay)".to_string(),
                category: "Solana / Smart Contracts".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Manual deserialization with try_from_slice() or raw account access without validating Anchor 8-byte discriminator or account owner: allows arbitrary account structure substitution.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-SOLANA-MISSING-SIGNER-CHECK".to_string(),
                cve_id: "CWE-287 / SOLANA-MISSING-SIGNER-CHECK".to_string(),
                name: "Solana: Privileged Authority Key Verification without is_signer Assertion".to_string(),
                category: "Solana / Smart Contracts".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Authority or admin public key comparison (authority.key == ...) without enforcing account.is_signer check: allows unauthorized users to pass arbitrary admin public keys without signing.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-UNSOUND-TRANSMUTE-ALIGNMENT".to_string(),
                cve_id: "CWE-843 / RUST-UNSOUND-TRANSMUTE-ALIGNMENT".to_string(),
                name: "Rust Soundness: Transmute or Raw Pointer Cast Violating Type Memory Alignment".to_string(),
                category: "Rust / Memory Safety".to_string(),
                severity: "HIGH".to_string(),
                description: "Transmute or slice::from_raw_parts between types of differing alignment requirements without alignment assertion (align_of::<T>()): causes undefined behavior, SIMD crashes, or dead code elimination by LLVM.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION".to_string(),
                cve_id: "CWE-662 / RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION".to_string(),
                name: "Rust Async: Cancellation-Unsafe Operation in tokio::select! Branch".to_string(),
                category: "Rust / Concurrency".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Non-cancellation-safe future (write_all, read_exact, or multi-step network framing) inside tokio::select! branch: cancellation leaves stream in corrupted/half-written state.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-COSMWASM-UNBOUNDED-STORAGE-ITERATION".to_string(),
                cve_id: "CWE-400 / COSMWASM-SUBSTRATE-STORAGE-BOMB".to_string(),
                name: "CosmWasm / Substrate: Unbounded KV-Storage Prefix Iteration (Gas Freeze DoS)".to_string(),
                category: "CosmWasm / Substrate".to_string(),
                severity: "HIGH".to_string(),
                description: "Iterating storage map or prefix with .range() or .prefix() without strict .take(LIMIT) pagination: state spam causes block gas limit exhaustion, permanently freezing smart contract state.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-EVM-L2-DATA-GAS-UNDERESTIMATION".to_string(),
                cve_id: "CWE-682 / EVM-L2-DATA-GAS-UNDERESTIMATION".to_string(),
                name: "Layer 2: Unaccounted L1 Calldata / Blob Fee in Relayer Gas Calculation".to_string(),
                category: "Layer 2 / Rollups".to_string(),
                severity: "HIGH".to_string(),
                description: "L2 transaction refund or subsidization calculating cost solely via gasleft() * tx.gasprice without querying L1 GasPriceOracle: calldata-heavy transactions drain relayer treasury at zero cost.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-ERC4337-PAYMASTER-STORAGE-POISONING".to_string(),
                cve_id: "CWE-667 / ERC4337-PAYMASTER-STORAGE-POISONING".to_string(),
                name: "ERC-4337 Account Abstraction: Paymaster Global Storage Rule Violation (Mempool DoS)".to_string(),
                category: "Web3 / Account Abstraction".to_string(),
                severity: "HIGH".to_string(),
                description: "Access or mutation of global unassociated storage in validatePaymasterUserOp: triggers ERC-4337 bundler reputation slashing and banishment of paymaster from alternative mempools.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-ARBITRARY-FILE-WRITE-TRAVERSAL".to_string(),
                cve_id: "CWE-22 / RUST-ARBITRARY-FILE-WRITE-TRAVERSAL".to_string(),
                name: "Écriture de Fichier Arbitraire / Path Traversal vers RCE (Rust)".to_string(),
                category: "SAST / Injection & RCE".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Écriture ou création de fichier (File::create, fs::write, jemalloc prof.dump) sur un chemin dérivé d'une entrée réseau/HTTP sans validation canonique stricte (.canonicalize() / .starts_with()) : permet d'écraser des fichiers système sensibles (cron.d, systemd, shell rc, DLL/.so) menant à l'exécution de code arbitraire distante (RCE).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-COMMAND-INJECTION-SHELL".to_string(),
                cve_id: "CWE-78 / RUST-COMMAND-INJECTION-SHELL".to_string(),
                name: "Injection de Commande Système OS via Interpréteur Shell (sh -c / cmd /C)".to_string(),
                category: "SAST / Injection & RCE".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de processus système via Command::new invoquant un interpréteur shell (sh, bash, cmd, powershell) avec des drapeaux d'évaluation (-c, /C) et des variables non sanitisées : vecteur direct d'exécution de commande arbitraire distante.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-UNSOUND-SLICE-FROM-RAW-PARTS".to_string(),
                cve_id: "CWE-119 / RUST-UNSOUND-SLICE-FROM-RAW-PARTS".to_string(),
                name: "Corruption Mémoire: slice::from_raw_parts sur Pointeurs Non Vérifiés".to_string(),
                category: "Rust / Memory Safety & RCE".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Reconstitution de structures de données ou tranches mémoire via slice::from_raw_parts ou Vec::from_raw_parts sans assertion sur la nullité (.is_null()), l'alignement ou les limites d'allocation : permet l'écrasement de pointeurs de saut ou vtables vers RCE.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-UNBOUNDED-BCS-DESERIALIZE".to_string(),
                cve_id: "CWE-502 / RUST-UNBOUNDED-BCS-DESERIALIZE".to_string(),
                name: "Désérialisation Non Bornée BCS / Serde (Crash Récursif & Gadgets)".to_string(),
                category: "SAST / Deserialization".to_string(),
                severity: "HIGH".to_string(),
                description: "Désérialisation directe bcs::from_bytes sans contrôle de taille de buffer ni garde-fou sur la récursion : risque de Stack Overflow non rattrapable ou manipulation de structures Move.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-MOVE-NATIVE-UNSAFE-BUFFER-BOUNDS".to_string(),
                cve_id: "CWE-787 / MOVE-NATIVE-UNSAFE-BUFFER-BOUNDS".to_string(),
                name: "Move VM Native Extension: Manipulation Mémoire Unsafe sans Contrôle de Bornes".to_string(),
                category: "Move VM / Memory Safety & RCE".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Dans une fonction native Move en Rust (aptos_framework / native), manipulation de pointeurs bruts (copy_nonoverlapping, .offset()) avec des longueurs ou indices issus de transactions Move sans clamp ni validation stricte : faille d'évasion de sandbox VM vers RCE host.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-DYNAMIC-LIB-HIJACK".to_string(),
                cve_id: "CWE-427 / RUST-DYNAMIC-LIB-HIJACK".to_string(),
                name: "Chargement Dynamique de Bibliothèque Non Sécurisé (DLL / .so Hijacking)".to_string(),
                category: "SAST / Injection & RCE".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Chargement de module ou bibliothèque partagée dynamique (.so / .dll via libloading ou dlopen) avec un chemin relatif ou non protégé : exécution de code arbitraire par substitution de bibliothèque locale.".to_string(),
                default_enabled: true,
            },
            // --- 7 NOUVELLES DÉTECTIONS RUST ULTRA BAS-NIVEAU (COMPILER / ABI / UNWIND / KERNEL) ---
            CveRuleDef {
                rule_id: "CVE-RUST-LLVM-DEAD-STORE-SECRET-LEAK".to_string(),
                cve_id: "CWE-14 / LLVM-DSE-SECRET-ZEROIZATION-ELIMINATION".to_string(),
                name: "Rust Compiler / LLVM: Secret Zeroization Stripped by Dead Store Elimination (CWE-14)".to_string(),
                category: "Rust / Compiler & Cryptography".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Effacement manuel d'une clé privée, seed ou secret en mémoire via .fill(0), ptr::write_bytes(..., 0, ...) ou assignation [0u8; N] sans zeroize::Zeroize ni ptr::write_volatile + compiler_fence : la passe Dead Store Elimination (DSE) de LLVM supprime l'écriture en mode --release car la variable n'est plus lue, laissant le secret intact dans la RAM.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB".to_string(),
                cve_id: "CWE-415 / RUST-UNWIND-PREMATURE-SET-LEN-DOUBLE-FREE".to_string(),
                name: "Rust Unwind Safety: Premature Vec::set_len or Unprotected ptr::read Drop-Bomb (Double-Free / Uninit Drop UB)".to_string(),
                category: "Rust / Memory Safety & RCE".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Appel à Vec::set_len(n) avant le remplissage effectif du tampon (read_exact, copy_nonoverlapping, boucle faillible) ou ptr::read() sans ManuallyDrop / garde RAII : en cas de panic! ou de retour anticipé '?', le déroulement de pile (stack unwind) exécute Drop sur de la mémoire non initialisée ou libère deux fois la même allocation (Double-Free vers RCE).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB".to_string(),
                cve_id: "CWE-908 / RUST-UNINIT-ASSUME-INIT-AND-STRUCT-PADDING-BLEED".to_string(),
                name: "Rust ABI & Memory: MaybeUninit::uninit().assume_init() Poison UB & Struct Alignment Padding Memory Leak".to_string(),
                category: "Rust / Memory Safety & ABI".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Instanciation d'un tampon via MaybeUninit::uninit().assume_init() / mem::uninitialized() (poison LLVM immédiat supprimant les contrôles de bornes), ou conversion d'une struct en &[u8] via slice::from_raw_parts(..., size_of::<T>()) sans bytemuck::NoUninit ni #[repr(packed)] : les octets de padding d'alignement divulguent des pointeurs ASLR/clés de la pile sur le réseau ou brisent le déterminisme des hachages de consensus.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-ALLOC-LAYOUT-ZERO-OR-ISIZE-OVERFLOW".to_string(),
                cve_id: "CWE-680 / RUST-ALLOCATOR-ZERO-LAYOUT-AND-ISIZE-OVERFLOW-UB".to_string(),
                name: "Rust Global Allocator: Zero-Sized Layout Allocation UB & from_size_align_unchecked Rounding Overflow".to_string(),
                category: "Rust / Allocator & Memory Safety".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Appel direct à std::alloc::alloc / alloc_zeroed sans garantir layout.size() > 0 (comportement indéfini immédiat en Rust contrairement à malloc(0)), ou construction de Layout::from_size_align_unchecked(size, align) sans vérifier size <= isize::MAX - (align - 1), provoquant un débordement d'arrondi dans jemalloc/mimalloc et un Heap Buffer Overflow.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-FFI-PANIC-ACROSS-ABI-BOUNDARY".to_string(),
                cve_id: "CWE-248 / RUST-EXTERN-C-UNWIND-ACROSS-FFI-UB".to_string(),
                name: "Rust FFI / Wasm Host ABI: Uncaught Panic Unwinding Across extern \"C\" Boundary".to_string(),
                category: "Rust / ABI & FFI Soundness".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Fonction extern \"C\" fn (sans ABI \"C-unwind\") exécutant des opérations susceptibles de paniquer (.unwrap(), .expect(), assert!(), indexation directe) sans confinement std::panic::catch_unwind : la traversée d'une frontière FFI C/C++/Wasm par un panic Rust corrompt les tables d'unwind ABI (UB) ou déclenche un abort() immédiat du nœud validateur.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-UNSAFECELL-CONCURRENT-MUT-NOALIAS-UB".to_string(),
                cve_id: "CWE-362 / RUST-LLVM-NOALIAS-UNSAFECELL-MUT-UB".to_string(),
                name: "Rust LLVM Codegen: Unsynchronized &mut T from UnsafeCell::get() Violating LLVM noalias".to_string(),
                category: "Rust / Concurrency & LLVM Soundness".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Matérialisation d'une référence exclusive &mut *cell.get() depuis un UnsafeCell dans une méthode partagée (&self) ou un type Sync sans synchronisation atomique / verrou exclusif : rustc émet l'attribut LLVM noalias sur tout &mut T, poussant l'optimiseur -O3 à supprimer les relectures mémoire et corrompre silencieusement l'état concurrent.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-MMAP-EXTERNAL-TRUNCATION-SIGBUS".to_string(),
                cve_id: "CWE-367 / RUST-MMAP-SLICE-TOCTOU-SIGBUS-AND-UB".to_string(),
                name: "Rust OS / Storage: Unlocked File Mmap::map Exposed as &[u8] (Hardware SIGBUS Crash & LLVM Jump-Table UB)".to_string(),
                category: "Rust / OS Kernel & Storage Soundness".to_string(),
                severity: "HIGH".to_string(),
                description: "Projection d'un fichier en mémoire via memmap2::Mmap::map exposée en &[u8] sans verrouillage exclusif OS (flock / lock_exclusive) : une troncature externe du fichier déclenche un signal matériel noyau SIGBUS non rattrapable par catch_unwind (crash instantané du validateur), et toute mutation concurrente viole l'immuabilité de &[u8] dans les tables de saut LLVM.".to_string(),
                default_enabled: true,
            },
        ]
    }

    /// Audit syntaxique & heuristique pour les 10 règles élite
    pub fn audit_file(
        file_path: &str,
        content: &str,
        is_test_file: bool,
        is_interface_or_types: bool,
    ) -> Vec<LinterViolation> {
        let mut violations = Vec::new();
        let lower_path = file_path.to_lowercase();
        if is_test_file || is_interface_or_types || lower_path.contains("rules.rs") || lower_path.contains("deep_rules.rs") {
            return violations;
        }

        let lines: Vec<&str> = content.lines().collect();
        let is_rust = lower_path.ends_with(".rs");
        let is_sol = lower_path.ends_with(".sol") || lower_path.ends_with(".vy");
        let is_circom = lower_path.ends_with(".circom");

        // ---------------------------------------------------------------------
        // 1. CVE-ZK-WEAK-FIAT-SHAMIR (CWE-347) - Version portée de fonction chirurgicale
        // ---------------------------------------------------------------------
        let is_zk_context = (is_rust || is_sol)
            && (lower_path.contains("transcript")
                || lower_path.contains("prover")
                || lower_path.contains("verifier")
                || lower_path.contains("fiat_shamir")
                || lower_path.contains("zk")
                || lower_path.contains("snark")
                || content.contains("challenge_scalar")
                || content.contains("squeeze_challenge")
                || content.contains("get_challenge")
                || content.contains("squeeze_scalar")
                || content.contains("FiatShamir"));

        if is_zk_context {
            let mut public_bound_in_scope = false;
            let mut recent_absorb_line: Option<usize> = None;

            for (idx, raw_line) in lines.iter().enumerate() {
                let l = raw_line.trim().to_lowercase();
                if l.starts_with("//") || l.starts_with("/*") || l.starts_with('*') || l.starts_with('#') {
                    continue;
                }

                // Découpage léger par portée de fonction (évite qu'un prove() sain masque un verify() vulnérable)
                if l.starts_with("pub fn ")
                    || l.starts_with("pub async fn ")
                    || l.starts_with("fn ")
                    || l.starts_with("async fn ")
                    || l.starts_with("function ")
                {
                    public_bound_in_scope = false;
                    recent_absorb_line = None;
                }

                // Détection de l'appel d'absorption
                if l.contains("append") || l.contains("absorb") || l.contains("observe") || l.contains("update") {
                    recent_absorb_line = Some(idx);
                }

                // Fenêtre de 4 lignes pour capter les arguments formatés par rustfmt / solfmt
                if let Some(abs_idx) = recent_absorb_line {
                    if idx >= abs_idx
                        && idx - abs_idx <= 4
                        && (l.contains("public") || l.contains("instance") || l.contains("vk") || l.contains("statement") || l.contains("pi"))
                    {
                        public_bound_in_scope = true;
                    }
                }

                // Détection du premier défi dans CETTE fonction
                let is_challenge = l.contains("challenge_scalar")
                    || l.contains("squeeze_challenge")
                    || l.contains("get_challenge")
                    || l.contains("squeeze_scalar")
                    || (l.contains("hash_to_scalar") && l.contains("challenge"));

                if is_challenge && !public_bound_in_scope {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: idx + 1,
                        rule_id: "CVE-ZK-WEAK-FIAT-SHAMIR".to_string(),
                        cve_id: "CWE-347 / WEAK-FIAT-SHAMIR-TRANSCRIPT-BYPASS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-347] WEAK_FIAT_SHAMIR : Défi Fiat-Shamir échantillonné dans cette fonction avant l'absorption explicite des entrées publiques (public_inputs / instance / vk) dans le Transcript : faille Frozen Heart ZK".to_string(),
                        transmitted_data: Some("unbound_fiat_shamir_transcript".to_string()),
                        transmitted_type: Some("Transcript / Merlin (Unbound Challenge Sampling)".to_string()),
                        remediation: "Absorber toutes les déclarations publiques et engagements dans le transcript via transcript.append_message(b'public_inputs', &public_inputs) AVANT d'appeler challenge_scalar().".to_string(),
                    });
                    // On marque true pour ne pas spammer sur les défis suivants (beta, gamma...) de la même fonction
                    public_bound_in_scope = true;
                }
            }
        }

        // ---------------------------------------------------------------------
        // 2. CVE-ZK-UNDERCONSTRAINED-SIGNAL (CWE-682)
        // ---------------------------------------------------------------------
        if is_circom || (is_rust && (lower_path.contains("circuit") || content.contains("ConstraintSystem"))) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                // Circom assignment operator <-- without subsequent === constraint on next 5 lines
                if is_circom && trimmed.contains("<--") && !trimmed.contains("===") {
                    let var_name = trimmed.split("<--").next().unwrap_or("").trim();
                    let has_r1cs_constraint = !var_name.is_empty() && lines.iter().skip(idx).take(6).any(|l| {
                        let t = l.trim();
                        !t.starts_with("//") && !t.starts_with("/*") && !t.starts_with('*') && t.contains("===") && t.contains(var_name)
                    });

                    if !has_r1cs_constraint {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "CVE-ZK-UNDERCONSTRAINED-SIGNAL".to_string(),
                            cve_id: "CWE-682 / ZK-UNDERCONSTRAINED-CIRCUIT".to_string(),
                            severity: "CRITICAL".to_string(),
                            message: format!("☣️ [CWE-682] ZK_UNDERCONSTRAINED : Witness assignment operator '<--' used for signal '{}' without corresponding '===' R1CS constraint: witness values are not enforced by the circuit", var_name),
                            transmitted_data: Some("unconstrained_zk_signal".to_string()),
                            transmitted_type: Some("Signal (Unchecked Circom Witness Generation)".to_string()),
                            remediation: "Replace '<--' with '<==' or add an explicit 'signal === expression' constraint directly following assignment.".to_string(),
                        });
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 3. CVE-DEFI-READ-ONLY-REENTRANCY (CWE-841)
        // ---------------------------------------------------------------------
        if is_sol && (lower_path.contains("vault") || lower_path.contains("lending") || lower_path.contains("oracle") || lower_path.contains("collateral") || lower_path.contains("liquidat")) {
            let has_pool_interaction = content.contains("get_virtual_price()")
                || content.contains(".getRate()")
                || content.contains("get_rate()")
                || content.contains(".getPoolTokens(");

            if has_pool_interaction {
                for (idx, &line) in lines.iter().enumerate() {
                    let line_no = idx + 1;
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                        continue;
                    }

                    if (trimmed.contains(".get_virtual_price()") || trimmed.contains(".getRate()") || trimmed.contains(".getPoolTokens("))
                        && !content.contains("nonReentrant")
                        && !content.contains("ReentrancyGuard")
                        && !content.contains("pool.claim_admin_fees")
                        && !content.contains("lock")
                        && !content.contains("checkReentrancy")
                    {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "CVE-DEFI-READ-ONLY-REENTRANCY".to_string(),
                            cve_id: "CWE-841 / DEFI-READ-ONLY-REENTRANCY".to_string(),
                            severity: "CRITICAL".to_string(),
                            message: "☣️ [CWE-841] DEFI_READ_ONLY_REENTRANCY : AMM view price feed (get_virtual_price/getRate) queried without reentrancy guard or pool reentrancy lock check: vulnerable to flash loan read-only reentrancy liquidation".to_string(),
                            transmitted_data: Some("amm_virtual_price_feed".to_string()),
                            transmitted_type: Some("uint256 (Reentrant Pool State Calculation)".to_string()),
                            remediation: "Apply nonReentrant modifier or verify the pool's reentrancy lock before consuming virtual prices in lending and collateral calculations.".to_string(),
                        });
                        break;
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 4. CVE-SOLANA-ACCOUNT-CONFUSION-DISCRIMINATOR (CWE-843)
        // ---------------------------------------------------------------------
        if is_rust && (lower_path.contains("solana") || lower_path.contains("program") || lower_path.contains("processor") || content.contains("AccountInfo")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                let is_manual_deser = trimmed.contains("try_from_slice(&")
                    || trimmed.contains("::deserialize(&mut")
                    || trimmed.contains("unpack_from_slice(");

                if is_manual_deser
                    && !content.contains("DISCRIMINATOR")
                    && !content.contains("discriminator")
                    && !content.contains("Account<'info,")
                    && !content.contains("owner ==")
                    && !content.contains("owner !=")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-SOLANA-ACCOUNT-CONFUSION-DISCRIMINATOR".to_string(),
                        cve_id: "CWE-843 / SOLANA-ACCOUNT-CONFUSION-DISCRIMINATOR".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-843] SOLANA_ACCOUNT_CONFUSION : Manual account deserialization without verifying 8-byte Anchor discriminator or account owner: vulnerable to account cosplay / type confusion attack".to_string(),
                        transmitted_data: Some("unvalidated_solana_account".to_string()),
                        transmitted_type: Some("AccountInfo (Missing 8-Byte Type Discriminator Check)".to_string()),
                        remediation: "Enforce discriminator verification: require!(account.data.borrow()[..8] == ExpectedStruct::DISCRIMINATOR) or use Anchor's Account<'info, T> wrapper.".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 5. CVE-SOLANA-MISSING-SIGNER-CHECK (CWE-287)
        // ---------------------------------------------------------------------
        if is_rust && (lower_path.contains("solana") || lower_path.contains("instruction") || lower_path.contains("processor") || content.contains("AccountInfo")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                let is_auth_check = (trimmed.contains(".key ==") || trimmed.contains(".key() ==") || trimmed.contains("== expected_authority"))
                    && (trimmed.contains("authority") || trimmed.contains("admin") || trimmed.contains("owner") || trimmed.contains("payer"));

                if is_auth_check
                    && !content.contains("is_signer")
                    && !content.contains("Signer<'info>")
                    && !content.contains("Signer<")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-SOLANA-MISSING-SIGNER-CHECK".to_string(),
                        cve_id: "CWE-287 / SOLANA-MISSING-SIGNER-CHECK".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-287] SOLANA_MISSING_SIGNER : Authority key comparison without verifying account.is_signer: an attacker can supply the admin public key without providing a cryptographic signature".to_string(),
                        transmitted_data: Some("unsigned_solana_authority".to_string()),
                        transmitted_type: Some("AccountInfo.key (Unsigned Key Match Authorization)".to_string()),
                        remediation: "Assert account.is_signer: require!(authority.is_signer, ProgramError::MissingRequiredSignature) or use Anchor's Signer<'info> account type.".to_string(),
                    });
                    break;
                }
            }
        }

        // ---------------------------------------------------------------------
        // 6. CVE-RUST-UNSOUND-TRANSMUTE-ALIGNMENT (CWE-843)
        // ---------------------------------------------------------------------
        if is_rust {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                let is_transmute_cast = (trimmed.contains("std::mem::transmute(") || trimmed.contains("mem::transmute(") || trimmed.contains("slice::from_raw_parts("))
                    && (trimmed.contains("*const u64") || trimmed.contains("*const u128") || trimmed.contains("&[u64]") || trimmed.contains("&[u128]") || trimmed.contains("&Header") || trimmed.contains("from_raw_parts"));

                if is_transmute_cast
                    && !trimmed.contains("align_of")
                    && !content.contains("align_to")
                    && !content.contains("align_of")
                    && !content.contains("is_aligned")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-UNSOUND-TRANSMUTE-ALIGNMENT".to_string(),
                        cve_id: "CWE-843 / RUST-UNSOUND-TRANSMUTE-ALIGNMENT".to_string(),
                        severity: "HIGH".to_string(),
                        message: "☣️ [CWE-843] RUST_UNSOUND_ALIGNMENT : Raw pointer transmute or slice::from_raw_parts across differing type alignments without alignment check: causes Undefined Behavior and memory faults".to_string(),
                        transmitted_data: Some("unaligned_pointer_transmute".to_string()),
                        transmitted_type: Some("Raw Pointer (Unaligned Memory Layout Cast)".to_string()),
                        remediation: "Ensure pointer alignment before casting via ptr.align_offset(align_of::<T>()) == 0, or use bytemuck::try_cast_slice().".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 7. CVE-RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION (CWE-662)
        // ---------------------------------------------------------------------
        if is_rust && content.contains("select!") {
            let mut in_select_block = false;
            let mut select_depth = 0;

            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                if trimmed.contains("tokio::select!") || trimmed.contains("select! {") {
                    in_select_block = true;
                    select_depth = 0;
                }

                if in_select_block {
                    if trimmed.contains('{') {
                        select_depth += trimmed.matches('{').count();
                    }
                    if trimmed.contains('}') {
                        select_depth = select_depth.saturating_sub(trimmed.matches('}').count());
                        if select_depth == 0 {
                            in_select_block = false;
                        }
                    }

                    let is_cancellation_unsafe = trimmed.contains(".write_all(")
                        || trimmed.contains(".read_exact(")
                        || trimmed.contains(".send_all(")
                        || trimmed.contains("framed.send(");

                    if is_cancellation_unsafe {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "CVE-RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION".to_string(),
                            cve_id: "CWE-662 / RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION".to_string(),
                            severity: "CRITICAL".to_string(),
                            message: "☣️ [CWE-662] ASYNC_CANCELLATION_CORRUPTION : Cancellation-unsafe operation (write_all/read_exact) evaluated inside tokio::select! branch: dropping intermediate future leaves stream in corrupted framing state".to_string(),
                            transmitted_data: Some("select_cancellation_future".to_string()),
                            transmitted_type: Some("AsyncStream (Non-Atomic Multi-Step Frame Incomplete)".to_string()),
                            remediation: "Buffer complete frames in memory before sending, or decouple the stream writer into a dedicated background task communicating via mpsc channel.".to_string(),
                        });
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 8. CVE-COSMWASM-UNBOUNDED-STORAGE-ITERATION (CWE-400) - Fenêtre chaînée rustfmt
        // ---------------------------------------------------------------------
        if is_rust && (content.contains("cw_storage_plus") || content.contains("cosmwasm_std") || lower_path.contains("contract")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                // Début de l'itération de range / prefix
                if (trimmed.contains(".range(") || trimmed.contains(".prefix("))
                    && !content.contains("MAX_LIMIT")
                    && !content.contains("DEFAULT_LIMIT")
                {
                    // Scanner la fenêtre de 8 lignes de l'expression chaînée jusqu'à `;` ou `.collect`
                    let window_end = (idx + 9).min(lines.len());
                    let chained_lines = &lines[idx..window_end];

                    let mut has_take = false;
                    let mut reaches_terminator = false;

                    for &w_line in chained_lines {
                        let wl = w_line.trim();
                        if wl.starts_with("//") || wl.starts_with("/*") || wl.starts_with('*') {
                            continue;
                        }
                        if wl.contains(".take(") {
                            has_take = true;
                            break;
                        }
                        if wl.contains(';') || wl.contains(".collect::<") || wl.contains(".collect()") {
                            reaches_terminator = true;
                        }
                    }

                    if !has_take && reaches_terminator {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "CVE-COSMWASM-UNBOUNDED-STORAGE-ITERATION".to_string(),
                            cve_id: "CWE-400 / COSMWASM-SUBSTRATE-STORAGE-BOMB".to_string(),
                            severity: "HIGH".to_string(),
                            message: "☣️ [CWE-400] COSMWASM_STORAGE_BOMB : Storage range iteration without mandatory .take(LIMIT) bounds: state spam forces transaction gas exhaustion, permanently locking contract state".to_string(),
                            transmitted_data: Some("unbounded_storage_range".to_string()),
                            transmitted_type: Some("StorageIterator (Uncapped Block Gas Consumption)".to_string()),
                            remediation: "Always apply .take(limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)) to all storage range iterations.".to_string(),
                        });
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 9. CVE-EVM-L2-DATA-GAS-UNDERESTIMATION (CWE-682) - Scope de fonction Solidity
        // ---------------------------------------------------------------------
        if is_sol && (lower_path.contains("l2") || lower_path.contains("rollup") || lower_path.contains("relay") || content.contains("Arbitrum") || content.contains("Optimism") || content.contains("Base")) {
            let has_l1_oracle = content.contains("getL1Fee")
                || content.contains("GasPriceOracle")
                || content.contains("ArbGasInfo")
                || content.contains("l1BaseFee");

            if !has_l1_oracle {
                let mut in_function = false;
                let mut fn_has_gasleft = false;

                for (idx, &line) in lines.iter().enumerate() {
                    let line_no = idx + 1;
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                        continue;
                    }

                    if trimmed.starts_with("function ") {
                        in_function = true;
                        fn_has_gasleft = false;
                    }

                    if in_function {
                        if trimmed.contains("gasleft()") {
                            fn_has_gasleft = true;
                        }

                        let is_gas_price_usage = trimmed.contains("tx.gasprice") || trimmed.contains("block.basefee");

                        if fn_has_gasleft && is_gas_price_usage {
                            violations.push(LinterViolation {
                                file_path: file_path.to_string(),
                                line_number: line_no,
                                rule_id: "CVE-EVM-L2-DATA-GAS-UNDERESTIMATION".to_string(),
                                cve_id: "CWE-682 / EVM-L2-DATA-GAS-UNDERESTIMATION".to_string(),
                                severity: "HIGH".to_string(),
                                message: "☣️ [CWE-682] L2_DATA_GAS_THEFT : L2 gas fee reimbursement computed without factoring L1 calldata / blob overhead: allows relayer treasury draining with calldata-heavy transactions".to_string(),
                                transmitted_data: Some("l2_unaccounted_l1_fee".to_string()),
                                transmitted_type: Some("uint256 (Missing L1 Data Gas Price Component)".to_string()),
                                remediation: "Query the L2 GasPriceOracle contract (e.g., OVM_GasPriceOracle.getL1Fee(msg.data)) and add the L1 data cost to total reimbursed gas.".to_string(),
                            });
                            fn_has_gasleft = false; // Une seule alerte par fonction
                        }
                    }

                    if trimmed == "}" {
                        in_function = false;
                        fn_has_gasleft = false;
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 10. CVE-ERC4337-PAYMASTER-STORAGE-POISONING (CWE-667) - Tracking d'accolades strict
        // ---------------------------------------------------------------------
        if is_sol && (content.contains("IPaymaster") || content.contains("validatePaymasterUserOp") || lower_path.contains("paymaster")) {
            let mut in_validate_paymaster = false;
            let mut brace_depth = 0;

            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                if trimmed.contains("function validatePaymasterUserOp") {
                    in_validate_paymaster = true;
                    brace_depth = 0;
                }

                if in_validate_paymaster {
                    brace_depth += trimmed.matches('{').count();
                    let close_count = trimmed.matches('}').count();

                    if brace_depth > 0 {
                        let is_global_storage_write = (trimmed.contains("totalGasUsed +=")
                            || trimmed.contains("totalGasUsed =")
                            || trimmed.contains("globalCounter++")
                            || trimmed.contains("lastOperationTimestamp =")
                            || (trimmed.contains(" = ")
                                && !trimmed.contains("userOp.sender")
                                && !trimmed.contains("memory ")
                                && !trimmed.contains("uint")
                                && !trimmed.contains("bytes")
                                && !trimmed.contains("address ")
                                && !trimmed.contains("bool ")))
                            && !trimmed.contains("return ");

                        if is_global_storage_write {
                            violations.push(LinterViolation {
                                file_path: file_path.to_string(),
                                line_number: line_no,
                                rule_id: "CVE-ERC4337-PAYMASTER-STORAGE-POISONING".to_string(),
                                cve_id: "CWE-667 / ERC4337-PAYMASTER-STORAGE-POISONING".to_string(),
                                severity: "HIGH".to_string(),
                                message: "☣️ [CWE-667] ERC4337_PAYMASTER_STORAGE_POISONING : Global unassociated storage write inside validatePaymasterUserOp: violates ERC-4337 validation rules and causes bundler reputation banishment (mempool DoS)".to_string(),
                                transmitted_data: Some("erc4337_unassociated_storage".to_string()),
                                transmitted_type: Some("StorageSlot (Global Unpartitioned Paymaster Write)".to_string()),
                                remediation: "Only access storage associated with userOp.sender in validatePaymasterUserOp or defer global state updates to postOp().".to_string(),
                            });
                        }
                    }

                    brace_depth = brace_depth.saturating_sub(close_count);
                    if brace_depth == 0 && trimmed.contains('}') {
                        in_validate_paymaster = false;
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 11. CVE-RUST-ARBITRARY-FILE-WRITE-TRAVERSAL (CWE-22)
        // ---------------------------------------------------------------------
        if is_rust && (content.contains("File::create") || content.contains("fs::write") || content.contains("prof.dump") || content.contains("jemalloc_ctl::raw::write")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }
                let is_file_write = trimmed.contains("File::create(")
                    || trimmed.contains("fs::write(")
                    || (trimmed.contains("raw::write(") && content.contains("prof.dump"));

                if is_file_write
                    && !content.contains("canonicalize()")
                    && !content.contains(".starts_with(")
                    && !content.contains("components()")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-ARBITRARY-FILE-WRITE-TRAVERSAL".to_string(),
                        cve_id: "CWE-22 / RUST-ARBITRARY-FILE-WRITE-TRAVERSAL".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-22] ARBITRARY_FILE_WRITE : Écriture de fichier ou dump mémoire sans vérification canonique (.canonicalize() / .starts_with()) : risque de Path Traversal permettant l'écrasement ou la création de fichiers système menant à une RCE".to_string(),
                        transmitted_data: Some("unvalidated_output_path".to_string()),
                        transmitted_type: Some("Path / File (Path Traversal to RCE)".to_string()),
                        remediation: "Valider le chemin cible via canonicalize() et vérifier qu'il commence strictement par le répertoire de destination autorisé (canonical.starts_with(base_dir)). Interdire tout '..' et slashs initiaux.".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 12. CVE-RUST-COMMAND-INJECTION-SHELL (CWE-78)
        // ---------------------------------------------------------------------
        if is_rust && content.contains("Command::new") {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }
                let is_shell_invoc = (trimmed.contains("Command::new(\"sh\")")
                    || trimmed.contains("Command::new(\"bash\")")
                    || trimmed.contains("Command::new(\"cmd\")")
                    || trimmed.contains("Command::new(\"powershell\")"))
                    && (content.contains(".arg(\"-c\")") || content.contains(".arg(\"/C\")") || content.contains(".arg(\"-Command\")"));

                if is_shell_invoc {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-COMMAND-INJECTION-SHELL".to_string(),
                        cve_id: "CWE-78 / RUST-COMMAND-INJECTION-SHELL".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-78] OS_COMMAND_INJECTION : Invocation directe d'un shell (sh -c / cmd /C) avec arguments potentiellement interpolés : risque immédiat d'exécution de code arbitraire (RCE)".to_string(),
                        transmitted_data: Some("shell_process_execution".to_string()),
                        transmitted_type: Some("Command / Process (OS Shell Execution)".to_string()),
                        remediation: "Ne pas invoquer de shell intermédiaire (sh/bash/cmd). Appeler directement l'exécutable binaire avec Command::new(\"binary\").args([arg1, arg2]) sans passage par un interpréteur shell.".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 13. CVE-RUST-UNSOUND-SLICE-FROM-RAW-PARTS (CWE-119)
        // ---------------------------------------------------------------------
        if is_rust && (content.contains("from_raw_parts") || content.contains("from_raw_parts_mut")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }
                let is_raw_parts = trimmed.contains("slice::from_raw_parts(")
                    || trimmed.contains("slice::from_raw_parts_mut(")
                    || trimmed.contains("Vec::from_raw_parts(");

                if is_raw_parts
                    && !content.contains("assert!(!")
                    && !content.contains(".is_null()")
                    && !content.contains("align_of")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-UNSOUND-SLICE-FROM-RAW-PARTS".to_string(),
                        cve_id: "CWE-119 / RUST-UNSOUND-SLICE-FROM-RAW-PARTS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-119] MEMORY_CORRUPTION_SLICE : Reconstitution de slice ou vecteur brut sans vérification de nullité (.is_null()) ni contrôle d'alignement mémoire : risque d'Out-of-Bounds Write / RCE".to_string(),
                        transmitted_data: Some("raw_memory_slice".to_string()),
                        transmitted_type: Some("RawPointer / Slice (Unbounded Pointer Dereference)".to_string()),
                        remediation: "Valider impérativement que le pointeur brut est non-nul, correctement aligné et que la longueur ne dépasse pas le buffer alloué.".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 14. CVE-RUST-UNBOUNDED-BCS-DESERIALIZE (CWE-502)
        // ---------------------------------------------------------------------
        if is_rust && (content.contains("bcs::from_bytes") || (content.contains("bcs::") && content.contains("deserialize"))) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }
                if trimmed.contains("bcs::from_bytes(")
                    && !content.contains("MAX_RECURSION")
                    && !content.contains("MAX_DEPTH")
                    && !content.contains("MAX_SIZE")
                    && !content.contains("limit")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-UNBOUNDED-BCS-DESERIALIZE".to_string(),
                        cve_id: "CWE-502 / RUST-UNBOUNDED-BCS-DESERIALIZE".to_string(),
                        severity: "HIGH".to_string(),
                        message: "☣️ [CWE-502] UNBOUNDED_DESERIALIZATION : Désérialisation bcs::from_bytes directe sans limiteur de profondeur ou de taille de buffer : risque d'épuisement de pile récursif (Stack Overflow) ou DoS / RCE".to_string(),
                        transmitted_data: Some("unbounded_bcs_payload".to_string()),
                        transmitted_type: Some("BCS / SerializedBytes (Unbounded Recursion)".to_string()),
                        remediation: "Imposer un plafond strict sur la taille du buffer avant désérialisation ou utiliser un décodeur avec limiteur de profondeur récursive.".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 15. CVE-MOVE-NATIVE-UNSAFE-BUFFER-BOUNDS (CWE-787)
        // ---------------------------------------------------------------------
        if is_rust && (lower_path.contains("native") || lower_path.contains("framework") || content.contains("NativeResult") || content.contains("NativeContext")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }
                let is_unsafe_ptr_op = trimmed.contains("copy_nonoverlapping")
                    || trimmed.contains(".offset(")
                    || (trimmed.contains(".as_mut_ptr()") && trimmed.contains("unsafe"));

                if is_unsafe_ptr_op && !content.contains("assert!(") && !content.contains("bounds_check") && !content.contains("bail!(") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-MOVE-NATIVE-UNSAFE-BUFFER-BOUNDS".to_string(),
                        cve_id: "CWE-787 / MOVE-NATIVE-UNSAFE-BUFFER-BOUNDS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-787] MOVE_NATIVE_MEM_CORRUPTION : Manipulation de mémoire brute dans une fonction native Move sans vérification des bornes : évasion de sandbox VM vers RCE de l'hôte".to_string(),
                        transmitted_data: Some("move_native_raw_buffer".to_string()),
                        transmitted_type: Some("NativePointer / FFI (Unbounded Native Write)".to_string()),
                        remediation: "Valider strictement les offsets et longueurs par rapport à la taille réelle des buffers avant toute opération mémoire unsafe.".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 16. CVE-RUST-DYNAMIC-LIB-HIJACK (CWE-427)
        // ---------------------------------------------------------------------
        if is_rust && (content.contains("libloading::Library") || content.contains("dlopen(") || content.contains("LoadLibrary")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }
                let is_lib_load = trimmed.contains("Library::new(") || trimmed.contains("dlopen(") || trimmed.contains("LoadLibraryW(");

                if is_lib_load && !content.contains("canonicalize()") && !content.contains("is_absolute()") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-DYNAMIC-LIB-HIJACK".to_string(),
                        cve_id: "CWE-427 / RUST-DYNAMIC-LIB-HIJACK".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-427] DYNAMIC_LIB_HIJACK : Chargement de bibliothèque partagée dynamique (.so / .dll) sans validation de chemin absolu : risque de DLL Hijacking et exécution de code arbitraire (RCE)".to_string(),
                        transmitted_data: Some("dynamic_library_path".to_string()),
                        transmitted_type: Some("Path / SharedLibrary (DLL / SO Preload Hijack)".to_string()),
                        remediation: "Exiger un chemin absolu vérifié (path.is_absolute()) issu d'un répertoire système ou d'installation protégé en écriture.".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 17. CVE-RUST-LLVM-DEAD-STORE-SECRET-LEAK (CWE-14 / LLVM DSE Zeroization Bypass)
        // ---------------------------------------------------------------------
        if is_rust
            && (content.contains(".fill(0")
                || content.contains("write_bytes(")
                || content.contains("= [0u8;")
                || content.contains("= [0;"))
            && !content.contains("zeroize")
            && !content.contains("Zeroize")
            && !content.contains("write_volatile")
            && !content.contains("compiler_fence")
            && !content.contains("black_box")
        {
            let mut in_drop_fn = false;
            let mut drop_brace_depth = 0usize;

            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                if trimmed.contains("fn drop(&mut self") || trimmed.contains("impl Drop for") {
                    in_drop_fn = true;
                    drop_brace_depth = 0;
                }

                if in_drop_fn {
                    drop_brace_depth += trimmed.matches('{').count();
                }

                let lower_line = trimmed.to_lowercase();
                let mentions_secret = lower_line.contains("secret")
                    || lower_line.contains("priv_key")
                    || lower_line.contains("private_key")
                    || lower_line.contains("privkey")
                    || lower_line.contains("seed")
                    || lower_line.contains("mnemonic")
                    || lower_line.contains("scalar")
                    || lower_line.contains("key_bytes")
                    || lower_line.contains("signing_key")
                    || lower_line.contains("master_key");

                let is_naive_zeroing = trimmed.contains(".fill(0)")
                    || trimmed.contains(".fill(0u8)")
                    || (trimmed.contains("write_bytes(") && trimmed.contains(", 0,"))
                    || ((!trimmed.starts_with("let ") && !trimmed.starts_with("const ") && !trimmed.starts_with("static "))
                        && (trimmed.contains("= [0u8;") || trimmed.contains("= [0;")));

                if is_naive_zeroing && (in_drop_fn || mentions_secret) {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-LLVM-DEAD-STORE-SECRET-LEAK".to_string(),
                        cve_id: "CWE-14 / LLVM-DSE-SECRET-ZEROIZATION-ELIMINATION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-14] LLVM_DEAD_STORE_SECRET_LEAK : Effacement manuel de clé/secret (.fill(0) / write_bytes / = [0u8; N]) sans zeroize ni write_volatile : l'optimiseur LLVM (Dead Store Elimination) supprime cette écriture en --release car le buffer n'est plus relu".to_string(),
                        transmitted_data: Some("unzeroized_secret_buffer".to_string()),
                        transmitted_type: Some("[u8] / SecretKey (LLVM DSE Elided Memory Wipe)".to_string()),
                        remediation: "Utiliser la crate zeroize (secret.zeroize() ou #[derive(Zeroize, ZeroizeOnDrop)]) qui combine ptr::write_volatile et atomic::compiler_fence(Ordering::SeqCst).".to_string(),
                    });
                }

                if in_drop_fn {
                    let close_count = trimmed.matches('}').count();
                    drop_brace_depth = drop_brace_depth.saturating_sub(close_count);
                    if drop_brace_depth == 0 && close_count > 0 {
                        in_drop_fn = false;
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 18. CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB (CWE-415 / Premature set_len & ptr::read Drop-Bomb)
        // ---------------------------------------------------------------------
        if is_rust && (content.contains(".set_len(") || content.contains("ptr::read(")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                let has_safety_comment = lines[idx.saturating_sub(4)..(idx + 2).min(lines.len())]
                    .iter()
                    .any(|l| l.to_uppercase().contains("// SAFETY:") || l.contains("ManuallyDrop") || l.contains("scopeguard") || l.contains("DropGuard"));

                // Cas A : .set_len(new_len) appelé AVANT le remplissage du buffer (read_exact, copy_nonoverlapping, write)
                if trimmed.contains(".set_len(")
                    && !trimmed.contains(".set_len(0)")
                    && !has_safety_comment
                {
                    let lookahead_end = (idx + 12).min(lines.len());
                    let mut filled_after_set_len = false;
                    for &next_line in &lines[(idx + 1)..lookahead_end] {
                        let nt = next_line.trim();
                        if nt.starts_with("pub fn ") || nt.starts_with("fn ") || nt.starts_with("pub async fn ") || nt.starts_with("async fn ") {
                            break;
                        }
                        if nt.contains(".read_exact(")
                            || nt.contains(".read(")
                            || nt.contains("copy_nonoverlapping(")
                            || nt.contains(".as_mut_ptr()")
                            || nt.contains("ptr::write(")
                        {
                            filled_after_set_len = true;
                            break;
                        }
                    }

                    if filled_after_set_len {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB".to_string(),
                            cve_id: "CWE-415 / RUST-UNWIND-PREMATURE-SET-LEN-DOUBLE-FREE".to_string(),
                            severity: "CRITICAL".to_string(),
                            message: "☣️ [CWE-415] UNWIND_DROP_BOMB_UB : Appel prématuré à Vec::set_len() AVANT l'initialisation complète du tampon : en cas d'erreur (?) ou de panic!, le déroulement de pile (unwind) exécute Drop sur de la mémoire non initialisée (UB / RCE)".to_string(),
                            transmitted_data: Some("uninit_vec_drop_on_unwind".to_string()),
                            transmitted_type: Some("Vec<T> (Premature set_len Before Fallible Write)".to_string()),
                            remediation: "Appeler Vec::set_len() UNIQUEMENT après le succès de l'écriture (read_exact / copy_nonoverlapping), ou utiliser Vec::spare_capacity_mut().".to_string(),
                        });
                    }
                }

                // Cas B : ptr::read() suivi d'une opération pouvant paniquer sans ManuallyDrop ni ajustement préalable de longueur
                if (trimmed.contains("ptr::read(") || trimmed.contains("std::ptr::read(") || trimmed.contains("core::ptr::read("))
                    && !has_safety_comment
                    && !content.contains("ManuallyDrop")
                    && !content.contains("mem::forget(")
                {
                    let lookahead_end = (idx + 8).min(lines.len());
                    let has_panicking_op_before_guard = lines[(idx + 1)..lookahead_end].iter().any(|l| {
                        let nt = l.trim();
                        (nt.contains(".clone()")
                            || nt.contains(".unwrap()")
                            || nt.contains("panic!(")
                            || nt.contains("assert!(")
                            || nt.contains("user_fn(")
                            || nt.contains("callback(")
                            || nt.contains("f("))
                            && !nt.contains(".set_len(")
                    });

                    if has_panicking_op_before_guard {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB".to_string(),
                            cve_id: "CWE-415 / RUST-UNWIND-PREMATURE-SET-LEN-DOUBLE-FREE".to_string(),
                            severity: "CRITICAL".to_string(),
                            message: "☣️ [CWE-415] UNWIND_DOUBLE_FREE_UB : Duplication bit-à-bit via ptr::read() suivie d'un appel susceptible de paniquer sans ManuallyDrop : un panic! déclenche un Double-Free lors du stack unwinding".to_string(),
                            transmitted_data: Some("duplicated_owned_ptr_read".to_string()),
                            transmitted_type: Some("*const T (Unprotected ptr::read Across Unwind Point)".to_string()),
                            remediation: "Encapsuler la valeur lue dans ManuallyDrop<T> ou décrémenter la taille du conteneur (set_len) AVANT d'appeler ptr::read() ou tout callback externe.".to_string(),
                        });
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 19. CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB (CWE-908 / Uninit Poison & Struct Padding Bleed)
        // ---------------------------------------------------------------------
        if is_rust
            && (content.contains("assume_init(")
                || content.contains("mem::uninitialized(")
                || (content.contains("from_raw_parts(") && content.contains("size_of::<")))
        {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                // Variante 1 : MaybeUninit::uninit().assume_init() ou mem::uninitialized()
                let prev_window = &lines[idx.saturating_sub(3)..=idx];
                let has_uninit_right_before = prev_window.iter().any(|l| l.contains("MaybeUninit::uninit()") || l.contains("MaybeUninit::<"))
                    && !prev_window.iter().any(|l| l.contains(".write(") || l.contains("as_mut_ptr()") || l.contains("copy_nonoverlapping"));

                let is_instant_uninit_ub = trimmed.contains("uninit().assume_init()")
                    || trimmed.contains("mem::uninitialized(")
                    || (trimmed.contains(".assume_init()") && has_uninit_right_before);

                // Variante 2 : Sérialisation brute d'une struct en &[u8] via from_raw_parts(..., size_of::<Struct>()) exposant les octets de padding
                let call_window = &lines[idx.saturating_sub(1)..(idx + 5).min(lines.len())];
                let is_raw_struct_padding_bleed = trimmed.contains("from_raw_parts(")
                    && call_window.iter().any(|l| l.contains("size_of::<") && !l.contains("size_of::<u8>"))
                    && call_window.iter().any(|l| l.contains("as *const u8"))
                    && !content.contains("bytemuck")
                    && !content.contains("NoUninit")
                    && !content.contains("zerocopy")
                    && !content.contains("repr(packed)");

                if is_instant_uninit_ub || is_raw_struct_padding_bleed {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB".to_string(),
                        cve_id: "CWE-908 / RUST-UNINIT-ASSUME-INIT-AND-STRUCT-PADDING-BLEED".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-908] UNINIT_POISON_AND_PADDING_BLEED : Appel à MaybeUninit::uninit().assume_init() (poison LLVM immédiat) ou exposition d'une struct en &[u8] via size_of::<T>() sans bytemuck::NoUninit (fuite des octets de padding d'alignement de la pile/heap sur le réseau)".to_string(),
                        transmitted_data: Some("uninitialized_padding_or_poison_bytes".to_string()),
                        transmitted_type: Some("MaybeUninit<T> / &[u8] (LLVM Poison Value or Struct Padding Leak)".to_string()),
                        remediation: "Ne jamais appeler assume_init() avant écriture complète (utiliser MaybeUninit::zeroed() ou ReadBuf), et utiliser #[derive(bytemuck::NoUninit)] + bytemuck::bytes_of() pour convertir une struct en &[u8].".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 20. CVE-RUST-ALLOC-LAYOUT-ZERO-OR-ISIZE-OVERFLOW (CWE-680 / Global Allocator Layout UB)
        // ---------------------------------------------------------------------
        if is_rust && (content.contains("from_size_align_unchecked(") || content.contains("alloc::alloc(") || content.contains("alloc_zeroed(")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                let has_safety_comment = lines[idx.saturating_sub(4)..=idx]
                    .iter()
                    .any(|l| l.to_uppercase().contains("// SAFETY:"));

                let is_unchecked_layout = trimmed.contains("from_size_align_unchecked(")
                    && !has_safety_comment
                    && !content.contains("isize::MAX")
                    && !content.contains("is_power_of_two()");

                let is_zero_size_alloc_ub = (trimmed.contains("std::alloc::alloc(")
                    || trimmed.contains("alloc::alloc(")
                    || trimmed.contains("alloc::alloc_zeroed(")
                    || trimmed.contains("std::alloc::alloc_zeroed("))
                    && !has_safety_comment
                    && !content.contains(".size() == 0")
                    && !content.contains(".size() > 0")
                    && !content.contains(".size() != 0")
                    && !content.contains("size == 0")
                    && !content.contains("size > 0")
                    && !content.contains("NonZeroUsize");

                if is_unchecked_layout || is_zero_size_alloc_ub {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-ALLOC-LAYOUT-ZERO-OR-ISIZE-OVERFLOW".to_string(),
                        cve_id: "CWE-680 / RUST-ALLOCATOR-ZERO-LAYOUT-AND-ISIZE-OVERFLOW-UB".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-680] ALLOCATOR_LAYOUT_UB : Allocation globale std::alloc::alloc sans garde layout.size() > 0 (UB immédiat en Rust) ou Layout::from_size_align_unchecked sans borne isize::MAX (débordement d'alignement vers Heap Buffer Overflow)".to_string(),
                        transmitted_data: Some("raw_allocator_layout".to_string()),
                        transmitted_type: Some("std::alloc::Layout (Zero-Size Alloc UB or Alignment Rounding Overflow)".to_string()),
                        remediation: "Utiliser Layout::from_size_align(size, align)? (qui vérifie align.is_power_of_two() et size <= isize::MAX - (align - 1)) et vérifier impérativement layout.size() > 0 avant d'appeler std::alloc::alloc().".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 21. CVE-RUST-FFI-PANIC-ACROSS-ABI-BOUNDARY (CWE-248 / Uncaught Unwind Across extern "C")
        // ---------------------------------------------------------------------
        if is_rust
            && (content.contains("extern \"C\"")
                || content.contains("extern \" \"")
                || content.contains("extern \"system\"")
                || content.contains("extern \"      \""))
            && !content.contains("catch_unwind")
        {
            let mut in_extern_c_fn = false;
            let mut extern_fn_line = 0usize;
            let mut brace_depth = 0usize;

            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                let is_extern_c_decl = (trimmed.contains("extern \"C\" fn ")
                    || trimmed.contains("extern \" \" fn ")
                    || trimmed.contains("extern \"system\" fn ")
                    || trimmed.contains("extern \"      \" fn "))
                    && !trimmed.contains("C-unwind")
                    && !trimmed.contains("extern \"        \"")
                    && !trimmed.ends_with(';');

                if is_extern_c_decl {
                    in_extern_c_fn = true;
                    extern_fn_line = line_no;
                    brace_depth = 0;
                }

                if in_extern_c_fn {
                    brace_depth += trimmed.matches('{').count();
                    let close_count = trimmed.matches('}').count();

                    let has_panic_trigger = trimmed.contains(".unwrap()")
                        || trimmed.contains(".expect(")
                        || trimmed.contains("panic!(")
                        || trimmed.contains("assert!(")
                        || trimmed.contains("assert_eq!(");

                    if has_panic_trigger {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "CVE-RUST-FFI-PANIC-ACROSS-ABI-BOUNDARY".to_string(),
                            cve_id: "CWE-248 / RUST-EXTERN-C-UNWIND-ACROSS-FFI-UB".to_string(),
                            severity: "CRITICAL".to_string(),
                            message: format!(
                                "☣️ [CWE-248] FFI_ABI_UNWIND_UB : Opération susceptible de paniquer (.unwrap/expect/assert/panic) dans la fonction FFI extern \"C\" (déclarée ligne {}) sans std::panic::catch_unwind : traverser une frontière ABI C/C++/Wasm par un panic provoque un UB ou un abort() immédiat du nœud",
                                extern_fn_line
                            ),
                            transmitted_data: Some("unconfined_ffi_panic_unwind".to_string()),
                            transmitted_type: Some("extern \"C\" StackFrame (Cross-ABI Unwind Undefined Behavior)".to_string()),
                            remediation: "Encapsuler l'intégralité du corps de la fonction extern \"C\" dans std::panic::catch_unwind(|| { ... }) et convertir toute panique en code d'erreur C.".to_string(),
                        });
                        in_extern_c_fn = false; // Une seule alerte par fonction FFI
                    } else {
                        brace_depth = brace_depth.saturating_sub(close_count);
                        if brace_depth == 0 && close_count > 0 {
                            in_extern_c_fn = false;
                        }
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // 22. CVE-RUST-UNSAFECELL-CONCURRENT-MUT-NOALIAS-UB (CWE-362 / LLVM noalias UnsafeCell::get() -> &mut T)
        // ---------------------------------------------------------------------
        if is_rust && content.contains(".get()") && content.contains("&mut *") {
            let has_sync_impl = content.contains("unsafe impl Sync") || content.contains("unsafe impl<") && content.contains("Sync for");
            let mut in_shared_self_fn = false;

            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                if trimmed.contains("fn ") && trimmed.contains("(&self") && !trimmed.contains("(&mut self") {
                    in_shared_self_fn = true;
                } else if trimmed.contains("fn ") && !trimmed.contains("(&self") {
                    in_shared_self_fn = false;
                }

                let has_safety_comment = lines[idx.saturating_sub(4)..=idx]
                    .iter()
                    .any(|l| l.to_uppercase().contains("// SAFETY:"));

                let is_unsafecell_mut_deref = trimmed.contains("&mut *")
                    && trimmed.contains(".get()")
                    && (in_shared_self_fn || has_sync_impl)
                    && !has_safety_comment
                    && !content.contains("compare_exchange")
                    && !content.contains("AtomicBool")
                    && !content.contains("AtomicUsize")
                    && !content.contains("AtomicU8");

                if is_unsafecell_mut_deref {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-UNSAFECELL-CONCURRENT-MUT-NOALIAS-UB".to_string(),
                        cve_id: "CWE-362 / RUST-LLVM-NOALIAS-UNSAFECELL-MUT-UB".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "☣️ [CWE-362] LLVM_NOALIAS_UNSAFECELL_UB : Déréférencement exclusif &mut *cell.get() depuis une méthode partagée (&self) ou un type Sync sans synchronisation atomique : viole l'attribut LLVM noalias et provoque des corruptions silencieuses en --release (-O3)".to_string(),
                        transmitted_data: Some("aliased_unsafecell_mut_ref".to_string()),
                        transmitted_type: Some("&mut T from UnsafeCell<T> (LLVM noalias Invariant Violation)".to_string()),
                        remediation: "Protéger l'accès par un protocole atomique d'exclusivité (AtomicBool::compare_exchange Acquire/Release), utiliser Mutex/RwLock, ou opérer via des pointeurs bruts (*mut T / ptr::write) sans matérialiser &mut T.".to_string(),
                    });
                }
            }
        }

        // ---------------------------------------------------------------------
        // 23. CVE-RUST-MMAP-EXTERNAL-TRUNCATION-SIGBUS (CWE-367 / Unlocked File Mmap SIGBUS & Jump-Table UB)
        // ---------------------------------------------------------------------
        if is_rust && (content.contains("Mmap::map(") || content.contains("MmapOptions::new()")) {
            for (idx, &line) in lines.iter().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                let has_safety_comment = lines[idx.saturating_sub(4)..=idx]
                    .iter()
                    .any(|l| l.to_uppercase().contains("// SAFETY:"));

                let is_mmap_call = trimmed.contains("Mmap::map(")
                    || (trimmed.contains(".map(") && content.contains("MmapOptions"));

                if is_mmap_call
                    && !has_safety_comment
                    && !content.contains("lock_exclusive")
                    && !content.contains("lock_shared")
                    && !content.contains("flock(")
                    && !content.contains("FileLock")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-MMAP-EXTERNAL-TRUNCATION-SIGBUS".to_string(),
                        cve_id: "CWE-367 / RUST-MMAP-SLICE-TOCTOU-SIGBUS-AND-UB".to_string(),
                        severity: "HIGH".to_string(),
                        message: "☣️ [CWE-367] MMAP_SIGBUS_AND_SLICE_UB : Projection mémoire Mmap::map exposée en &[u8] sans verrouillage OS (flock / lock_exclusive) : une troncature externe du fichier provoque un crash matériel SIGBUS non rattrapable par catch_unwind, et toute mutation externe viole l'immuabilité LLVM de &[u8]".to_string(),
                        transmitted_data: Some("unlocked_file_mmap_slice".to_string()),
                        transmitted_type: Some("memmap2::Mmap -> &[u8] (OS Page-Fault SIGBUS & Concurrent Aliasing UB)".to_string()),
                        remediation: "Verrouiller le descripteur de fichier via fs2::FileExt::lock_exclusive() avant Mmap::map(), et documenter l'absence de modification concurrente avec // SAFETY:.".to_string(),
                    });
                }
            }
        }

        violations
    }
}
