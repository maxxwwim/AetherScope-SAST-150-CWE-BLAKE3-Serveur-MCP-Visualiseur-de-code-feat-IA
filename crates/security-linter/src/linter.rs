use crate::contract::ReadmeContract;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterViolation {
    pub file_path: String,
    pub line_number: usize,
    pub rule_id: String,
    pub cve_id: String,
    pub severity: String, // "CRITICAL" | "HIGH" | "MEDIUM" | "WARNING"
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted_type: Option<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveRuleDef {
    pub rule_id: String,
    pub cve_id: String,
    pub name: String,
    pub category: String, // "Rust / Network" | "Web3 / Crypto" | "MMO / Concurrency" | "SAST / Injection" | "Architecture / Boundary" | "Front-End / DOM"
    pub severity: String,
    pub description: String,
    pub default_enabled: bool,
}

/// Vue du contenu d'un fichier qui mémorise le résultat des `contains` répétés.
struct ContentIndex<'a> {
    text: &'a str,
    cache: std::cell::RefCell<std::collections::HashMap<&'static str, bool>>,
}

impl<'a> ContentIndex<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, cache: std::cell::RefCell::new(std::collections::HashMap::new()) }
    }

    fn contains(&self, pat: &'static str) -> bool {
        if let Some(&v) = self.cache.borrow().get(pat) {
            return v;
        }
        let v = self.text.contains(pat);
        self.cache.borrow_mut().insert(pat, v);
        v
    }
}

impl<'a> std::ops::Deref for ContentIndex<'a> {
    type Target = str;
    fn deref(&self) -> &str {
        self.text
    }
}

pub struct SecurityLinter;

impl SecurityLinter {
    /// Catalogue exhaustif des règles et CVE prises en charge par AetherScope V2
    pub fn get_supported_cve_rules() -> Vec<CveRuleDef> {
        let mut rules = vec![
            // 0. Rust & Réseau Bas-Niveau (Failles Méconnues & Protocole)
            CveRuleDef {
                rule_id: "CVE-RUST-DESER-BOMB-OOM".to_string(),
                cve_id: "CWE-789 / LENGTH-PREFIX-OOM-BOMB".to_string(),
                name: "Bombe d'Allocation Réseau (Length-Prefix OOM Abort)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Désérialisation binaire (bincode, rmp_serde) ou Vec::with_capacity(wire_len) sans .with_limit() : un préfixe forgé de 8 octets provoque un abort() OOM immédiat.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-CANCEL-SAFETY-LOSS".to_string(),
                cve_id: "CWE-662 / TOKIO-SELECT-CANCEL-LOSS".to_string(),
                name: "Perte Silencieuse de Trame TCP (tokio::select! Cancel-Safety)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "HIGH".to_string(),
                description: "Utilisation de futures non cancel-safe (read_exact, write_all, read_to_end) dans tokio::select!, corrompant le flux TCP si une autre branche gagne.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-TIMING-SIDECHANNEL".to_string(),
                cve_id: "CWE-208 / TIMING-SIDE-CHANNEL".to_string(),
                name: "Fuite Temporelle Réseau (Non-Constant-Time Secret Compare)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "HIGH".to_string(),
                description: "Comparaison de secrets, nonces, HMAC ou signatures via == ou != (court-circuité par memcmp SIMD) au lieu de subtle::ConstantTimeEq::ct_eq().".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-NET-SLOWLORIS-KEEPALIVE-DOS".to_string(),
                cve_id: "CWE-400 / SLOWLORIS-SOCKET-EXHAUSTION".to_string(),
                name: "Épuisement de Sockets TCP/WS (Slowloris & Missing Timeout)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "HIGH".to_string(),
                description: "Acceptation de socket (accept_async, TcpListener::accept, axum::serve) sans tokio::time::timeout ni TimeoutLayer sur le handshake réseau.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-NET-SSRF-DNS-REBIND".to_string(),
                cve_id: "CWE-918 / SSRF-DNS-REBINDING".to_string(),
                name: "SSRF & DNS Rebinding TOCTOU sur Socket Sortant".to_string(),
                category: "Rust / Network".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Connexion sortante (reqwest, TcpStream::connect) vers une cible dynamique sans filtrage DNS post-résolution (is_loopback, is_private, 169.254.169.254).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-BLOCKING-IN-ASYNC".to_string(),
                cve_id: "CWE-400 / ASYNC-EXECUTOR-STARVATION".to_string(),
                name: "Famine du Runtime Async (I/O ou Sleep Bloquant dans async fn)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Appel bloquant synchrone (std::fs, std::thread::sleep, std::net::TcpStream) directement dans un contexte async fn sans tokio::task::spawn_blocking, paralysant les workers Tokio.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-INTEGER-OVERFLOW-CAST".to_string(),
                cve_id: "CWE-190 / AS-CAST-INTEGER-TRUNCATION".to_string(),
                name: "Troncature Silencieuse par Cast de Conversion Entière (as u32 / as u16 / as u8)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "HIGH".to_string(),
                description: "Conversion de taille de buffer, solde ou entier 64-bit avec l'opérateur 'as' sans contrôle de bornes, provoquant une troncature silencieuse des bits hauts en release.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-TRANSMUTE-SIZE-MISMATCH".to_string(),
                cve_id: "CWE-843 / UNSAFE-TRANSMUTE-TYPE-CONFUSION".to_string(),
                name: "Transmutation Arbitraire de Types Mémoire (std::mem::transmute)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Appel à std::mem::transmute() sans vérification de taille ou layout mémoire équivalent, entraînant un comportement indéfini (UB) et une corruption de mémoire immédiate.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-FORGOTTEN-MEM-LEAK".to_string(),
                cve_id: "CWE-775 / MEM-FORGET-RESOURCE-LEAK-DEADLOCK".to_string(),
                name: "Contournement RAII & Fuite Critique par std::mem::forget (Deadlock de Verrou)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "HIGH".to_string(),
                description: "Appel explicite à std::mem::forget() sur un garde de synchronisation (MutexGuard, RwLock) ou socket, empêchant le Drop de s'exécuter et causant des deadlocks permanents.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-UNCHECKED-INDEX-SLICING".to_string(),
                cve_id: "CWE-125 / UNCHECKED-SLICE-INDEX-PANIC-DOS".to_string(),
                name: "Indexation Directe de Tranche Réseau Sans Contrôle (Panic DOS sur Paquet Forgé)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "HIGH".to_string(),
                description: "Indexation directe par tranche [offset..offset + len] sur un buffer de trame réseau brut sans vérification .get(..) préalable, provoquant une panique instantanée du serveur sur paquet tronqué.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-THREAD-POISON-IGNORED".to_string(),
                cve_id: "CWE-391 / SWALLOWED-MUTEX-POISON-INVARIANT".to_string(),
                name: "Verrou Mutex Empoisonné Ignoré Silencieusement (Swallowed PoisonError)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "HIGH".to_string(),
                description: "Récupération aveugle d'un Mutex empoisonné via .unwrap_or_else(|e| e.into_inner()), continuant d'opérer sur des données partagées compromises par un thread paniqué.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-FFI-UNCHECKED-NULL-PTR".to_string(),
                cve_id: "CWE-476 / FFI-UNCHECKED-NULL-PTR-DEREF".to_string(),
                name: "Déréférencement FFI de Pointeur Brut Sans Vérification de Nullité".to_string(),
                category: "Rust / Network".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Déréférencement direct de pointeur brut (*const T / *mut T) dans un bloc unsafe sans assertion préalable ptr.is_null() ou sans NonNull, causant un crash SIGSEGV.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-PIN-STRUCTURAL-UNPIN-SOUNDNESS-UB".to_string(),
                cve_id: "RUST-INTERNALS-CRITICAL / PIN-GET-UNCHECKED-MUT-SOUNDNESS-BREAK".to_string(),
                name: "Rupture d'Invariant Pinning Async (Pin::get_unchecked_mut / Self-Referential Move UB)".to_string(),
                category: "Rust / Network".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Projection manuelle Pin::get_unchecked_mut ou Pin::new_unchecked sans pin_project! sur une Future ou structure auto-référentielle : un mem::swap ou déplacement invalide les pointeurs internes entre deux .await (Use-After-Free silencieux).".to_string(),
                default_enabled: true,
            },
            // 0.B L1 Blockchain Node (Consensus, P2P Borsh, Wasm Runtime, Fee Bypass, Crypto)
            CveRuleDef {
                rule_id: "CVE-L1-BORSH-UNBOUNDED-DESER-OOM".to_string(),
                cve_id: "L1-P2P-CONSENSUS-CRITICAL / BORSH-ZSTD-WIRE-ALLOCATION-BOMB".to_string(),
                name: "Bombe de Désérialisation P2P Borsh / Zstd sur Paquet ou StateWitness Non Authentifié".to_string(),
                category: "Rust / Network".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Désérialisation Borsh (BorshDeserialize::try_from_slice) ou décompression zstd::decode_all sur une trame P2P ou un ChunkStateWitness sans plafond de taille ni vérification préalable de signature : un préfixe u32::MAX ou une bombe zstd provoque un crash OOM de tous les validateurs.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-L1-RUNTIME-GAS-PREPAY-BYPASS".to_string(),
                cve_id: "L1-FEE-BYPASS-CRITICAL / HOST-FN-ALLOCATION-BEFORE-GAS-BURN".to_string(),
                name: "Contournement de Facturation de Gaz Wasm (Allocation Mémoire / I/O Avant burn_gas)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Fonction hôte du runtime Wasm (VMLogic / host functions) exécutant une allocation mémoire dynamique (vec![0; len], read_memory) ou un accès Trie avant d'avoir facturé le gaz proportionnel (gas_counter.burn_gas / pay_gas), permettant de geler un chunk producer gratuitement.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-L1-BALANCE-STORAGE-STAKING-DESYNC".to_string(),
                cve_id: "L1-FUNDS-CRITICAL / BALANCE-MUTATION-WITHOUT-STORAGE-STAKE-CHECK".to_string(),
                name: "Désynchronisation Balance / Storage Staking & Manipulation de Solde L1 (set_amount / saturating_sub)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Mutation directe du solde ou du stake d'un compte (set_amount, set_locked, saturating_sub sur Balance) sans safe_add_balance/checked_sub ou sans vérification finale check_storage_stake, permettant de siphonner les fonds réservés au stockage ou de créer une inflation par Refund.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-L1-CONSENSUS-UNVERIFIED-WITNESS-CHUNK".to_string(),
                cve_id: "L1-CONSENSUS-CRITICAL / STATELESS-WITNESS-UNVERIFIED-ENDORSEMENT".to_string(),
                name: "Traitement de ChunkStateWitness / Endorsement Avant Vérification Cryptographique du Validateur".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Application ou mise en cache d'un ChunkStateWitness, PartialEncodedChunk ou Approval dans le consensus Nightshade/Doomslug avant la validation de la signature du producteur de chunk/bloc, exposant au fork de consensus ou à l'empoisonnement d'état.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-L1-CROSS-SHARD-RECEIPT-NONCE-REPLAY".to_string(),
                cve_id: "L1-TX-CRITICAL / DELEGATE-ACTION-ACCESS-KEY-NONCE-BYPASS".to_string(),
                name: "Rejeu de Meta-Transaction NEP-366 (DelegateAction) & Contournement d'Allowance AccessKey".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de DelegateAction ou SignedTransaction sans vérification stricte de max_block_height, de la portée FunctionCallPermission ou de la mise à jour atomique de access_key.nonce avant l'émission de l'ActionReceipt.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-L1-CRYPTO-BLIND-SCALAR-MERKLE-MALLEABILITY".to_string(),
                cve_id: "L1-CRYPTO-CRITICAL / UNCHECKED-CURVE-DECOMPRESS-TORSION-SPLIT".to_string(),
                name: "Divergence de Consensus Cryptographique (Point BLS12-381 / Ed25519 / BN254 Non-Canonique ou Torsion)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Décompression de point elliptique ou vérification de signature/preuve Merkle via des primitives _unchecked (from_bytes_unchecked, decompress_unchecked) sans test de sous-groupe d'ordre premier (is_torsion_free / is_small_order), permettant un Consensus Split entre validateurs.".to_string(),
                default_enabled: true,
            },

            // 1. Web3 & Cryptographie
            CveRuleDef {
                rule_id: "CVE-WEB3-REENTRANCY-UNCHECKED".to_string(),
                cve_id: "CWE-841 / REENTRANCY".to_string(),
                name: "Anti-Réentrance Web3 & MMO".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Transfert d'actifs ou mise à jour de solde sans ReentrancyGuard ou Checks-Effects-Interactions.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-SIG-MALLEABILITY".to_string(),
                cve_id: "CWE-347 / ECDSA-MALLEABILITY".to_string(),
                name: "Malléabilité de Signature ECDSA (secp256k1)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Vérification de signature sans validation du s-value borné (s < secp256k1_n/2), ouvrant au rejeu.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-UNVALIDATED-CHAINID".to_string(),
                cve_id: "CWE-294 / CROSS-CHAIN-REPLAY".to_string(),
                name: "Rejeu de Signature Inter-Chaînes (Missing ChainId)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Challenge ou signature EIP-191/712 sans vérification du chain_id ou de contrat vérificateur.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-NONCE-RACE".to_string(),
                cve_id: "CWE-362 / TOCTOU-NONCE".to_string(),
                name: "Race Condition sur Nonce Web3".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "HIGH".to_string(),
                description: "Consommation ou suppression du nonce après l'action plutôt qu'avant (Time-Of-Check to Time-Of-Use).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-AUTH-BYPASS-MOCK".to_string(),
                cve_id: "CWE-287 / CRYPTO-SIGNATURE-BYPASS".to_string(),
                name: "Contournement Cryptographique & Signature Mock / Zero-Address".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Acceptation de signature factice (0xmock, bypass_sig) ou ecrecover sans rejet explicite de l'adresse nulle (0x0000...0000), permettant l'usurpation totale de n'importe quel wallet.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-ORACLE-SPOT-MANIPULATION".to_string(),
                cve_id: "CWE-682 / FLASH-LOAN-ORACLE-MANIPULATION".to_string(),
                name: "Manipulation d'Oracle Spot & Flash Loan AMM".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Lecture directe du cours sur les réserves spot d'une pool AMM (getReserves, slot0, balanceOf pair) sans oracle TWAP ni flux Chainlink avec contrôle de fraîcheur, ouvrant au vidage de fonds par Flash Loan.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-TX-ORIGIN-PHISHING".to_string(),
                cve_id: "CWE-302 / TX-ORIGIN-AUTHORIZATION-BYPASS".to_string(),
                name: "Contournement d'Accès par Hameçonnage (tx.origin vs msg.sender)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Authentification administrative ou autorisation de transfert via tx.origin au lieu de msg.sender, permettant à un contrat intermédiaire tiers d'usurper l'autorité du propriétaire.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-UNCHECKED-TRANSFER-RETURN".to_string(),
                cve_id: "CWE-252 / SILENT-TRANSFER-FAILURE-USDT".to_string(),
                name: "Échec Silencieux des Tokens Non-Standard (USDT/BNB Missing Return)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Appel de transfer() ou transferFrom() sans SafeERC20 : USDT et tokens non conformes ne renvoient pas de booléen, provoquant un revert systématique ou un échec silencieux créant des jetons gratuits.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-PERMIT-FRONT-RUNNING".to_string(),
                cve_id: "CWE-362 / MEMPOOL-SIGNATURE-GRIEFING".to_string(),
                name: "Vol & Griefing de Signature ERC-2612 Permit dans le Mempool".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "HIGH".to_string(),
                description: "Exécution de token.permit() sans try/catch dans une fonction de dépôt : un bot mempool peut front-runner la signature et provoquer un revert permanent (DoS/Griefing) pour l'utilisateur légitime.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-HOOK-REENTRANCY".to_string(),
                cve_id: "CWE-841 / ERC777-ERC1363-HOOK-REENTRANCY".to_string(),
                name: "Réentrance Invisible via Hooks ERC-777 / ERC-1363 (tokensReceived)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Transfert de tokens ERC-20 sans ReentrancyGuard : les tokens compatibles ERC-777 ou ERC-1363 invoquent un callback automatique sur le destinataire, ouvrant une réentrance même sans envoi d'ETH.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-VM-ACCOUNT-MISSING-OWNER".to_string(),
                cve_id: "CWE-284 / VM-ACCOUNT-OWNER-CONFUSION".to_string(),
                name: "Confusion de Compte VM (Missing Account Owner Check)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Programme VM à modèle de comptes en Rust acceptant un AccountInfo sans vérifier que account.owner == program_id, permettant à un attaquant de passer un compte forgé par un programme tiers pour voler des actifs.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-READONLY-REENTRANCY".to_string(),
                cve_id: "CWE-841 / READONLY-REENTRANCY-LP-ORACLE".to_string(),
                name: "Réentrance en Lecture Seule sur Oracle LP (Curve / Balancer Virtual Price)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Interrogation du virtual_price ou de réserves d'une pool LP dans une fonction view/read-only non protégée pendant un retrait de liquidité en cours, lisant un cours temporairement faussé.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-DIRTY-MEMORY-ASSEMBLY".to_string(),
                cve_id: "CWE-682 / CWE-119-DIRTY-BITS-ASSEMBLY".to_string(),
                name: "Corruption de Mémoire Assembly Solidity (Dirty High-Order Bits)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Manipulation en inline assembly (mload, mstore) de types de moins de 32 octets (bool, address, uint8) sans masque binaire (and(val, 0xff)), polluant les bits hauts et inversant les conditions logiques.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-ERC4626-INFLATION-ATTACK".to_string(),
                cve_id: "CWE-682 / ERC4626-FIRST-DEPOSITOR-DONATION".to_string(),
                name: "Attaque par Inflation de Vault ERC-4626 (Donation Front-Running)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Calcul de parts dans un Vault ERC-4626 sans parts virtuelles (_decimalsOffset) : le premier déposant mint 1 wei de part puis fait une donation massive, tronquant à 0 les dépôts des utilisateurs suivants.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-VAULT-ROUNDING-DIRECTION".to_string(),
                cve_id: "CWE-682 / INCORRECT-CONVERSION-ROUNDING".to_string(),
                name: "Direction d'Arrondi Inversée ERC-4626 / Vault (Micro-Minting Exploit)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Fonction de conversion ERC-4626 (convertToShares, convertToAssets, previewDeposit, etc.) utilisant une division brute / ou Rounding.Up au lieu de Rounding.Floor sur les dépôts : permet de voler des fractions d'actifs ou de gonfler indûment les parts.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-STALE-ORACLE-HEARTBEAT".to_string(),
                cve_id: "CWE-682 / STALE-ORACLE-ROUND-DATA".to_string(),
                name: "Oracle de Prix Sans Vérification de Fraîcheur / Heartbeat (Chainlink / Redstone)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "HIGH".to_string(),
                description: "Appel à latestRoundData() sans vérification de updatedAt == 0, block.timestamp - updatedAt > HEARTBEAT ou answeredInRound < roundId : risque d'utilisation de prix obsolètes lors de congestions réseau.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR".to_string(),
                cve_id: "CWE-284 / UNPROTECTED-FLASHLOAN-CALLBACK".to_string(),
                name: "Callback Flash Loan / Hook Sans Contrôle d'Accès msg.sender".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Fonction de callback (onFlashLoan, uniswapV3SwapCallback, etc.) publique ou externe sans vérification require(msg.sender == authorizedPool) : un attaquant externe peut invoquer directement le callback et drainer les autorisations de tokens.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-SIGNATURE-DEADLINE-MISSING".to_string(),
                cve_id: "CWE-613 / MISSING-SIGNATURE-DEADLINE".to_string(),
                name: "Signature EIP-712 / Permit Sans Date d'Expiration (Mempool Holding Attack)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "HIGH".to_string(),
                description: "Fonction consommant une signature cryptographique (ecrecover, ECDSA.recover) sans vérifier require(block.timestamp <= deadline) : des validateurs ou tiers peuvent stocker la signature et l'exécuter ultérieurement à un moment défavorable.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-HOOK-DEFERRED-CHECKS-DESYNC".to_string(),
                cve_id: "CWE-841 / TRANSIENT-LIQUIDITY-INVARIANT-VIOLATION".to_string(),
                name: "Désynchronisation EVC / ChecksDeferred dans HookTarget (Flash Invariant Bypass)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Hook de Vault ou contrat périphérique exécutant des transferts/appels externes ou lisant l'état de liquidité sans s'assurer que les vérifications EVC ne sont pas différées (require(!evc.isChecksDeferred()) ou require(evc.areChecksInProgress())).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-DECIMAL-SCALING-TRUNCATION".to_string(),
                cve_id: "CWE-682 / ORACLE-DECIMAL-SCALING-UNDERFLOW".to_string(),
                name: "Erreur de Normalisation de Décimales d'Oracle / Collatéral (Troncature & Underflow)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "HIGH".to_string(),
                description: "Mise à l'échelle de prix ou de collatéral supposant statiquement 18 décimales (ex: 10**(18 - decimals) ou division brute / 10**decimals) vulnérable aux tokens non-standard (WBTC 8 décimales, USDC 6 décimales, tokens > 18 décimales).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-FORK-REWARD-STAKING-DILUTION".to_string(),
                cve_id: "CWE-682 / REWARD-HARVEST-BEFORE-UPDATE".to_string(),
                name: "Dilution de Récompenses & Staking Reentrancy dans Fork DeFi (Points / Yield Farm)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Distribution ou récolte de récompenses (rewardToken.transfer, mint) lors d'un dépôt/retrait sans mise à jour préalable de l'accumulateur (accRewardPerShare) et du checkpoint temporel (lastRewardBlock).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-EIP1153-TRANSIENT-COMPOSABILITY-LEAK".to_string(),
                cve_id: "EVM-CANCUN-CRITICAL / TSTORE-UNCLEARED-COMPOSABILITY-POLLUTION".to_string(),
                name: "Pollution de Mémoire Transitoire EIP-1153 Inter-Appels (Dirty TSTORE en Multicall / Batch)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Écriture dans le stockage transitoire EVM (opcode tstore ou variable transient) sans remise à zéro explicite avant la fin de l'appel : dans un batch ou multicall, l'appel suivant hérite de l'état transitoire authentifié ou du delta de Flash Accounting.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-EIP7702-EOA-CODE-BYPASS-COLLISION".to_string(),
                cve_id: "EVM-PECTRA-CRITICAL / EIP7702-EXTCODESIZE-BYPASS-AND-SLOT-COLLISION".to_string(),
                name: "Contournement Anti-Contrat (extcodesize == 0 / code.length == 0) & Collision de Slots EIP-7702".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Vérification d'humanité/EOA basée sur msg.sender.code.length == 0 ou extcodesize == 0, ou contrat de délégation EOA EIP-7702 sans stockage isolé ERC-7201 : contournable par constructeur ou délégation atomique 0xef0100 et sujet à l'écrasement du slot 0.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-ERC4337-PAYMASTER-GAS-SIPHON".to_string(),
                cve_id: "AA-ERC4337-CRITICAL / PAYMASTER-POSTOP-UNBOUNDED-GAS-GRIEFING".to_string(),
                name: "Siphonnage de Trésorerie Paymaster ERC-4337 par Griefing de postOp & Gaz Non Borné".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Paymaster ERC-4337 validant validatePaymasterUserOp sans plafonner maxCost/postOpGasLimit ou effectuant un transferFrom ERC-20 uniquement dans postOp sans pré-séquestration : un utilisateur peut faire réverter son appel interne et vider le dépôt ETH du Paymaster sur l'EntryPoint.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-OMNICHAIN-LZ-CHANNEL-BRICKING".to_string(),
                cve_id: "OMNICHAIN-CRITICAL / CROSS-CHAIN-UNCAUGHT-REVERT-FUND-BURN".to_string(),
                name: "Destruction de Fonds Brûlés & Paralysie de Canal Cross-Chain (_lzReceive / _ccipReceive Sans Quarantaine)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Réception de message inter-chaînes (_lzReceive, lzReceive, _ccipReceive) exécutant un transfert ou callback externe sans isolation try/catch ni stockage failedMessages : tout revert sur la chaîne de destination détruit définitivement les tokens brûlés sur la chaîne source ou bloque le canal.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-EIP150-63-64-GAS-STIPEND-GRIEFING".to_string(),
                cve_id: "EVM-INTERNALS-CRITICAL / EIP150-63-64TH-SUBCALL-OOG-SILENT-BYPASS".to_string(),
                name: "Sabotage Silencieux de Sous-Appel par la Règle des 63/64 de Gaz EIP-150 (Relayer / MetaTx OOG Griefing)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution d'un sous-appel externe (.call ou try/catch) suivie de la validation d'un nonce ou du marquage d'un message comme exécuté sans vérifier gasleft() : l'EIP-150 ne transmet que 63/64 du gaz au sous-appel, permettant à un relayer de faire échouer l'appel interne par Out-Of-Gas tout en conservant 1/64 de gaz pour valider le nonce et voler la récompense.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-ZK-SNARK-FIELD-OVERFLOW-ALIASING".to_string(),
                cve_id: "ZK-CRYPTO-CRITICAL / BN254-SCALAR-FIELD-MODULO-ALIASING".to_string(),
                name: "Double-Spend de Preuve ZK-SNARK par Aliasing Modulo le Corps Scalaire BN254 (5x Nullifier Overflow)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Vérificateur ZK (Groth16 / Plonk / Circom) acceptant des entrées publiques ou nullifiers uint256 sans vérifier input[i] < SNARK_SCALAR_FIELD (r ≈ 2^254) : un attaquant ajoute k*r au nullifier pour rejouer 5 fois la même preuve ZK valide.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-MERKLE-SECOND-PREIMAGE-LEAF-NODE-COLLISION".to_string(),
                cve_id: "CRYPTO-CRITICAL / MERKLE-64BYTE-LEAF-INTERNAL-NODE-SPOOF".to_string(),
                name: "Falsification d'Arbre de Merkle par Seconde Pré-Image sur Feuille de 64 Octets (Leaf-Node Collision)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Hachage d'une feuille de Merkle via un simple keccak256(abi.encode(addr, amount)) de 64 octets sans double hachage bytes.concat(keccak256(...)) ni préfixe 0x00 : un attaquant passe un nœud interne de l'arbre (32+32 octets) comme feuille valide.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-WEB3-CREATE2-METAMORPHIC-REINIT-RUG".to_string(),
                cve_id: "EVM-BYTECODE-CRITICAL / CREATE2-UNSALTED-SENDER-FRONT-RUN-AND-METAMORPHIC".to_string(),
                name: "Détournement de Déploiement Déterministe CREATE2 (Salt Sans msg.sender & Bytecode Métamorphique)".to_string(),
                category: "Web3 / Crypto".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Déploiement CREATE2 utilisant un salt fourni par l'utilisateur sans dérivation keccak256(abi.encode(msg.sender, salt)) ou chargeant son initCode dynamiquement : vulnérable au front-running d'adresse mempool/cross-chain et à la substitution métamorphique.".to_string(),
                default_enabled: true,
            },

            // 2. MMO, Concurrence & Boucle de Jeu Rust
            CveRuleDef {
                rule_id: "CVE-RUST-ASYNC-MUTEX".to_string(),
                cve_id: "CWE-667 / ASYNC-DEADLOCK".to_string(),
                name: "Deadlock Asynchrone (std::sync::Mutex across .await)".to_string(),
                category: "MMO / Concurrency".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Garde de Mutex synchrone conservé à travers un point .await, bloquant le thread pool Tokio.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-UNBOUNDED-CHANNEL".to_string(),
                cve_id: "CWE-400 / UNBOUNDED-MEMORY-DOS".to_string(),
                name: "Saturation Mémoire Canal Réseau Non Borné".to_string(),
                category: "MMO / Concurrency".to_string(),
                severity: "HIGH".to_string(),
                description: "Utilisation de unbounded_channel() sur les flux WebSocket ou paquets réseau risquant le crash OOM.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-MMO-UNCHECKED-SPEEDHACK".to_string(),
                cve_id: "CWE-20 / UNCHECKED-SPEEDHACK".to_string(),
                name: "Téléportation & Speedhack Client Non Validé".to_string(),
                category: "MMO / Concurrency".to_string(),
                severity: "HIGH".to_string(),
                description: "Coordonnées de joueur assignées directement depuis un paquet client sans contrôle de vitesse max (delta_t).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-MATH-UNCHECKED-BALANCE".to_string(),
                cve_id: "CWE-190 / INTEGER-UNDERFLOW".to_string(),
                name: "Underflow Arithmétique de Solde / Monnaie".to_string(),
                category: "MMO / Concurrency".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Décrémentation de solde ou pièces sans saturating_sub ou checked_sub, provoquant un wrapping.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-MMO-IDOR-TRUSTED-USERID".to_string(),
                cve_id: "CWE-639 / IDOR-CLIENT-USERID-SPOOF".to_string(),
                name: "IDOR & Usurpation d'Identité Joueur via Payload Client (userId non authentifié)".to_string(),
                category: "MMO / Concurrency".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Extraction directe de userId / user_id depuis le corps de requête client sans résolution ni validation du token de session côté serveur (permet de déplacer/modifier un autre joueur).".to_string(),
                default_enabled: true,
            },

            // 3. SAST & Injections Fondamentales (Rust / Python / Redis / Multi-Lang)
            CveRuleDef {
                rule_id: "SAST_RAW_SQL_INJECTION".to_string(),
                cve_id: "CWE-89 / SQL-INJECTION".to_string(),
                name: "Injection SQL Brute Dynamique".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Concaténation dynamique de requêtes SQL dans les couches protégées au lieu de requêtes préparées.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "SAST_HARDCODED_SECRET".to_string(),
                cve_id: "CWE-798 / HARDCODED-SECRET".to_string(),
                name: "Identifiant ou Clé Secrète en Clair".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Clé privée, token API ou secret cryptographique hardcodé en clair dans le code source.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-DB-REDIS-UNAUTH-KEYS-DOS".to_string(),
                cve_id: "CWE-306 / CWE-400-REDIS-KEYS-DOS".to_string(),
                name: "DoS Redis O(N) (Commande KEYS *) & Socket Cache Non Authentifié".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de la commande bloquante O(N) KEYS sur Redis en production (au lieu de SCAN) ou exposition d'un serveur RESP/TCP sans authentification préalable.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-UNSAFE-MEM".to_string(),
                cve_id: "CWE-119 / CVE-RUST-UNSAFE-MEM".to_string(),
                name: "Bloc Unsafe & Pointeurs Bruts Rust".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Usage de unsafe, pointeurs bruts *mut T ou std::mem::transmute contournant le borrow-checker.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-PANIC-DOS".to_string(),
                cve_id: "CWE-248 / CVE-RUST-PANIC-DOS".to_string(),
                name: "Panique / Crash Process sur Entrée Invalide".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "HIGH".to_string(),
                description: ".unwrap() ou .expect() directs sur des entrées réseau/utilisateurs dans la couche serveur.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-RUST-CMD-INJ".to_string(),
                cve_id: "CWE-78 / CVE-RUST-CMD-INJ".to_string(),
                name: "Injection de Commande Système OS (Rust)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de processus via Command::new avec arguments non filtrés.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-DESERIALIZE-RCE".to_string(),
                cve_id: "CWE-502 / CVE-PY-DESERIALIZE-RCE".to_string(),
                name: "Désérialisation Python Non Sécurisée (Pickle/YAML/Eval)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de code arbitraire via pickle.loads(), yaml.load() sans SafeLoader ou eval().".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-SHELL-INJ".to_string(),
                cve_id: "CWE-78 / CVE-PY-SHELL-INJ".to_string(),
                name: "Injection Shell Python (subprocess shell=True / os.system)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de commande shell en Python avec shell=True ou os.system().".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-SSRF-UNVALIDATED".to_string(),
                cve_id: "CWE-918 / CVE-PY-SSRF-UNVALIDATED".to_string(),
                name: "Requête HTTP Sortante Non Filtrée (SSRF Python)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "HIGH".to_string(),
                description: "Requête requests/httpx vers une URL dynamique sans liste blanche ni blocage d'IP privées.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-UNTYPED-PAYLOAD".to_string(),
                cve_id: "CWE-20 / CVE-PY-UNTYPED-PAYLOAD".to_string(),
                name: "Payload API Python Non Typé (dict brut sans Pydantic)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "HIGH".to_string(),
                description: "Route API Python acceptant un dict brut au lieu d'un schéma Pydantic v2 strict.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-PATH-TRAVERSAL-ARCHIVE".to_string(),
                cve_id: "CWE-22 / PATH-TRAVERSAL-ZIPSLIP".to_string(),
                name: "Path Traversal & Extraction d'Archive Non Bornée (ZipSlip Python)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Ouverture de fichier dynamique (open, send_file) ou extraction d'archive (extractall, unpack_archive) sans vérification Path.resolve().is_relative_to(BASE_DIR).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-JINJA2-SSTI-INJECTION".to_string(),
                cve_id: "CWE-1336 / JINJA2-SSTI-RCE".to_string(),
                name: "Injection de Template Côté Serveur (Jinja2 SSTI RCE)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Concaténation de variables ou f-strings non échappées dans render_template_string() ou jinja2.Template(), permettant l'exécution de code arbitraire via l'exploration MRO.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-YAML-UNSAFE-LOAD".to_string(),
                cve_id: "CWE-502 / YAML-UNSAFE-DESERIALIZATION".to_string(),
                name: "Désérialisation Non Sécurisée PyYAML (yaml.load sans SafeLoader)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Appel à yaml.load() sans Loader=SafeLoader ou yaml.unsafe_load(), permettant à un document YAML forgé d'instancier des objets Python arbitraires et d'exécuter du code OS.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-SQL-ALCHEMY-TEXT-INJECTION".to_string(),
                cve_id: "CWE-89 / SQLALCHEMY-RAW-TEXT-INJECTION".to_string(),
                name: "Injection SQL Brute dans SQLAlchemy text() (Bypass ORM)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Utilisation de sqlalchemy.text() avec concaténation dynamique (f-string, +, %) au lieu de paramètres liés (:param / bindparams), réintroduisant les injections SQL dans les ORM.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-JWT-NONE-ALGORITHM".to_string(),
                cve_id: "CWE-327 / JWT-UNVERIFIED-NONE-ALGORITHM".to_string(),
                name: "Contournement d'Authentification JWT (Algorithme 'none' ou Signature Non Vérifiée)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Décodage de jeton JWT via jwt.decode() avec verify=False, verify_signature=False ou tolérance de l'algorithme 'none', permettant de forger n'importe quel token sans clé secrète.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-PARAMIKO-MISSING-HOST-KEY".to_string(),
                cve_id: "CWE-295 / PARAMIKO-AUTOADD-HOSTKEY-MITM".to_string(),
                name: "Connexion SSH Paramiko Vulnérable aux Attaques MITM (AutoAddPolicy)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "HIGH".to_string(),
                description: "Configuration de Paramiko avec AutoAddPolicy(), acceptant aveuglément n'importe quelle clé d'hôte SSH inconnue et exposant le trafic à une interception par Man-In-The-Middle.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-PY-XML-XXE-ENTITY-EXPANSION".to_string(),
                cve_id: "CWE-611 / XML-EXTERNAL-ENTITY-XXE".to_string(),
                name: "Injection d'Entités Externes XML (XXE / Entity Expansion)".to_string(),
                category: "SAST / Injection".to_string(),
                severity: "HIGH".to_string(),
                description: "Parsing XML avec ElementTree ou minidom sans désactiver la résolution des entités externes ou sans defusedxml, permettant la lecture de fichiers système ou le DoS Billion Laughs.".to_string(),
                default_enabled: true,
            },

            // 4. Architecture & Intégrité BLAKE3
            CveRuleDef {
                rule_id: "BLAKE3_TAMPERED".to_string(),
                cve_id: "CWE-353 / BLAKE3-INTEGRITY".to_string(),
                name: "Altération Clandestine de Contrat BLAKE3".to_string(),
                category: "Architecture / Boundary".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Fichier README.md ou contrat de dossier modifié sans signature officielle dans aether.lock.toml.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "RULE_FORBIDDEN_IMPORT".to_string(),
                cve_id: "CWE-1075 / ARCH-CONTRACT-VIOLATION".to_string(),
                name: "Franchissement Illégal de Frontière de Couche".to_string(),
                category: "Architecture / Boundary".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Import interdit violant l'étanchéité architecturale (ex: client important le domaine serveur).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "RULE_MAX_LINES".to_string(),
                cve_id: "CWE-1075 / ARCH-BUDGET-EXCEEDED".to_string(),
                name: "Dépassement de Budget de Lignes de Fichier".to_string(),
                category: "Architecture / Boundary".to_string(),
                severity: "WARNING".to_string(),
                description: "Le fichier dépasse la limite max_lines_per_file définie dans le contrat EARS du dossier.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "SAST_DIRECT_ENV_ACCESS".to_string(),
                cve_id: "CWE-15 / DIRECT-ENV-ACCESS".to_string(),
                name: "Accès Direct aux Variables d'Environnement".to_string(),
                category: "Architecture / Boundary".to_string(),
                severity: "HIGH".to_string(),
                description: "Accès à process.env ou std::env::var dans la couche domaine pur (100% sans I/O).".to_string(),
                default_enabled: true,
            },

            // 5. Front-End PixiJS / Node.js / TypeScript
            CveRuleDef {
                rule_id: "CVE-JS-EVENTLOOP-REDOS-JSON".to_string(),
                cve_id: "CWE-1333 / CWE-770-EVENTLOOP-DOS".to_string(),
                name: "Blocage de l'Event-Loop Node.js (Stream HTTP Non Borné & ReDoS)".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Accumulation de flux HTTP req.on('data') sans plafond d'octets (MAX_BODY_SIZE) ou RegExp dynamique non bornée, permettant de geler l'Event-Loop ou faire crasher V8.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-XSS-DOM".to_string(),
                cve_id: "CWE-79 / CVE-JS-XSS-DOM".to_string(),
                name: "Injection DOM / XSS dans l'UI".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "HIGH".to_string(),
                description: "Assignation brute via innerHTML, dangerouslySetInnerHTML ou document.write().".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-PROTO-POLLUTION".to_string(),
                cve_id: "CWE-1321 / CVE-JS-PROTO-POLLUTION".to_string(),
                name: "Pollution de Prototype / Payload Non Typé".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "HIGH".to_string(),
                description: "Ingestion de payload : any ou req.body sans validation de schéma strict (Zod/contracts).".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-CMD-INJ".to_string(),
                cve_id: "CWE-78 / CVE-JS-CMD-INJ".to_string(),
                name: "Injection de Commande Shell (Node.js)".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "CRITICAL".to_string(),
                description: "child_process.exec ou execSync exécutant des chaînes de commande dynamiques.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-CODE-INJ".to_string(),
                cve_id: "CWE-94 / CVE-JS-CODE-INJ".to_string(),
                name: "Évaluation Dynamique de Code (eval / new Function)".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Utilisation de eval() ou new Function() permettant l'exécution de code JavaScript arbitraire.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-CLIENT-UNSAFE-ASSETS".to_string(),
                cve_id: "CWE-829 / UNSAFE-ASSET-LOADING".to_string(),
                name: "Chargement d'Asset Graphique Non Validé".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "MEDIUM".to_string(),
                description: "Chargement dynamique de texture PixiJS / Three.js via Assets.load() sans domaine de confiance ou validation CDN.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-VM-SANDBOX-ESCAPE".to_string(),
                cve_id: "CWE-94 / VM-SANDBOX-ESCAPE-RCE".to_string(),
                name: "Échappement de Bac à Sable Node.js vm (Évasion de Sandbox & RCE)".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Exécution de code via le module natif vm (vm.runInContext, vm.runInNewContext, vm.Script). Le module vm n'est pas un mécanisme de sécurité et permet d'accéder au constructeur Function global pour exécuter du shell OS.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-SSRF-DYNAMIC-AXIOS-FETCH".to_string(),
                cve_id: "CWE-918 / AXIOS-FETCH-OUTBOUND-SSRF".to_string(),
                name: "SSRF Sortant via Requête HTTP Dynamique (axios / fetch / got)".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Envoi de requête HTTP sortante via axios, fetch, got ou http.get avec une URL dynamique non validée, permettant d'atteindre les métadonnées Cloud (169.254.169.254) ou des services internes locaux.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-PATH-TRAVERSAL-SENDFILE".to_string(),
                cve_id: "CWE-22 / SENDFILE-PATH-TRAVERSAL".to_string(),
                name: "Lecture Arbitraire de Fichier & Path Traversal (res.sendFile / fs.readFile)".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Transmission de fichier dynamique avec res.sendFile() ou fs.readFile() basé sur des entrées utilisateurs (req.params/req.query) sans confinement de racine, permettant d'extraire des fichiers système sensibles.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-REGEX-DOS-DYNAMIC-INPUT".to_string(),
                cve_id: "CWE-1333 / DYNAMIC-REGEXP-CATASTROPHIC-REDOS".to_string(),
                name: "Déni de Service ReDoS par Expression Régulière Dynamique".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "HIGH".to_string(),
                description: "Instanciation de new RegExp() directement depuis une variable utilisateur ou chaîne non échappée, permettant d'injecter des motifs à rebroussement exponentiel pour geler l'Event Loop.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-CORS-WILDCARD-CREDENTIALS".to_string(),
                cve_id: "CWE-942 / CORS-REFLECTED-ORIGIN-WITH-CREDENTIALS".to_string(),
                name: "Configuration CORS Dangereuse avec Origine Réfléchie & Identifiants".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "HIGH".to_string(),
                description: "Configuration CORS avec credentials: true associée à origin: true ou reflétant l'en-tête Origin client, permettant à un site web tiers malveillant d'exécuter des requêtes authentifiées avec cookies/jetons.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-UNHANDLED-PROMISE-CRASH".to_string(),
                cve_id: "CWE-755 / ASYNC-ROUTE-UNHANDLED-REJECTION".to_string(),
                name: "Rejet de Promesse Non Intercepté dans un Handler de Route Async (Crash Process)".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "HIGH".to_string(),
                description: "Handler de route Express/Koa async sans bloc try/catch et sans transfert vers next(err), déclenchant des UnhandledPromiseRejection qui provoquent l'arrêt immédiat du processus Node.js en production.".to_string(),
                default_enabled: true,
            },
            CveRuleDef {
                rule_id: "CVE-JS-V8-TYPEDARRAY-SHARED-AB-RACE-OOB".to_string(),
                cve_id: "V8-INTERNALS-CRITICAL / SHAREDARRAYBUFFER-TOCTOU-AND-ALLOCUNSAFE-BLEED".to_string(),
                name: "Fuite de Heap V8 (Buffer.allocUnsafe) & Race Condition TOCTOU sur SharedArrayBuffer".to_string(),
                category: "Front-End / DOM".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Allocation mémoire brute Buffer.allocUnsafe() sans .fill(0) exfiltrant les clés privées de la heap V8 sur le réseau (Nodebleed), ou lecture concurrente d'un SharedArrayBuffer sans Atomics.load permettant de modifier les bornes d'un paquet entre la validation et la lecture.".to_string(),
                default_enabled: true,
            },

            // 6. C / C++ & Modules Natifs FFI
            CveRuleDef {
                rule_id: "CVE-CPP-UNBOUNDED-COPY-UAF".to_string(),
                cve_id: "CWE-120 / BUFFER-OVERFLOW-FFI".to_string(),
                name: "Débordement de Tampon Mémoire C/C++ (strcpy / sprintf / memcpy non borné)".to_string(),
                category: "C / C++ / FFI".to_string(),
                severity: "CRITICAL".to_string(),
                description: "Utilisation de primitives C/C++ sans contrôle de taille (strcpy, strcat, sprintf, gets, memcpy non borné) exposant la pile ou le tas à une corruption mémoire critique.".to_string(),
                default_enabled: true,
            },
        ];
        rules.extend(crate::rules::AdvancedDefiRules::definitions());
        rules.extend(crate::deep_rules::DeepSecurityRules::definitions());
        rules
    }

    pub fn audit_file_with_rules(
        file_path: &str,
        content: &str,
        contract: Option<&ReadmeContract>,
        active_rules: Option<&HashSet<String>>,
    ) -> Vec<LinterViolation> {
        let all = Self::audit_file(file_path, content, contract);
        if let Some(allowed) = active_rules {
            all.into_iter()
                .filter(|v| allowed.contains(&v.rule_id))
                .collect()
        } else {
            all
        }
    }

    /// Règles dont la détection porte sur le contenu des littéraux de chaîne
    /// (secrets en dur, signatures mock, SQL concaténé...) : elles restent évaluées
    /// sur le code brut, les autres sur le code où strings/commentaires sont masqués.
    const LITERAL_AWARE_RULES: &'static [&'static str] = &[
        "SAST_HARDCODED_SECRET",
        "SAST_RAW_SQL_INJECTION",
        "CVE-WEB3-AUTH-BYPASS-MOCK",
        "SAST_DIRECT_ENV_ACCESS",
        "RULE_FORBIDDEN_IMPORT",
        "RULE_MAX_LINES",
        "CVE-DB-REDIS-UNAUTH-KEYS-DOS",
        "CVE-RUST-COMMAND-INJECTION-SHELL",
    ];

    /// Audit d'un fichier avec élimination des faux positifs issus de chaînes/commentaires :
    /// une violation n'est conservée que si elle est aussi détectée sur la version masquée
    /// du fichier (ou si la règle est sensible aux littéraux).
    pub fn audit_file(
        file_path: &str,
        content: &str,
        contract: Option<&ReadmeContract>,
    ) -> Vec<LinterViolation> {
        let raw = Self::audit_file_raw(file_path, content, contract);
        let lang = match crate::lexer::Lang::from_path(&file_path.to_lowercase()) {
            Some(l) => l,
            None => return raw,
        };
        if raw.is_empty() {
            return raw;
        }
        // Python: les règles SAST portent sur des littéraux (algorithms=["none"], Loader=...)
        // -> on ne masque que les commentaires. Autres langages: chaînes + commentaires.
        let masked_content = if lang == crate::lexer::Lang::Python {
            crate::lexer::mask_comments_only(content, lang)
        } else {
            crate::lexer::mask_literals(content, lang)
        };
        let masked: HashSet<(String, usize)> = Self::audit_file_raw(file_path, &masked_content, contract)
            .into_iter()
            .map(|v| (v.rule_id, v.line_number))
            .collect();
        raw.into_iter()
            .filter(|v| {
                Self::LITERAL_AWARE_RULES.contains(&v.rule_id.as_str())
                    || masked.contains(&(v.rule_id.clone(), v.line_number))
            })
            .collect()
    }

    /// Tronque le contenu d'un fichier Rust dès l'ouverture d'un module de test unitaire
    /// (`#[cfg(test)]`, `#[cfg(all(test...))]`, `mod tests {`, `mod test {`) afin qu'aucune
    /// règle (y compris `AdvancedDefiRules` et `DeepSecurityRules`) ne scanne les tests inline.
    pub fn strip_rust_inline_tests(content: &str) -> &str {
        let mut offset = 0usize;
        for line in content.split_inclusive('\n') {
            let trimmed = line.trim();
            if trimmed == "#[cfg(test)]"
                || trimmed.starts_with("#[cfg(all(test")
                || trimmed == "mod tests {"
                || trimmed == "mod test {"
            {
                return &content[..offset];
            }
            offset += line.len();
        }
        content
    }

    fn audit_file_raw(
        file_path: &str,
        content: &str,
        contract: Option<&ReadmeContract>,
    ) -> Vec<LinterViolation> {
        let mut violations = Vec::new();
        let lower_path = file_path.to_lowercase();
        let is_rust = lower_path.ends_with(".rs");
        let is_py = lower_path.ends_with(".py");
        let is_js_ts = lower_path.ends_with(".js")
            || lower_path.ends_with(".ts")
            || lower_path.ends_with(".tsx")
            || lower_path.ends_with(".jsx")
            || lower_path.ends_with(".mjs")
            || lower_path.ends_with(".cjs");
        let is_cpp = lower_path.ends_with(".c")
            || lower_path.ends_with(".cpp")
            || lower_path.ends_with(".cc")
            || lower_path.ends_with(".h")
            || lower_path.ends_with(".hpp");
        let is_web3_mmo = lower_path.contains("packet")
            || lower_path.contains("wallet")
            || lower_path.contains("trade")
            || lower_path.contains("inventory")
            || lower_path.contains("contract");
        let file_name = file_path
            .rsplit('/')
            .next()
            .unwrap_or(file_path)
            .rsplit('\\')
            .next()
            .unwrap_or(file_path);
        let lower_file_name = file_name.to_lowercase();

        let norm_path = lower_path.replace('\\', "/");

        // Exclusion réflexive : les catalogues de règles du linter ne doivent pas s'auto-signaler
        let is_linter_catalog = norm_path.contains("security-linter/src/")
            || lower_file_name == "linter.rs"
            || lower_file_name == "rules.rs"
            || lower_file_name == "deep_rules.rs"
            || lower_file_name == "lexer.rs";
        if is_linter_catalog {
            return violations;
        }

        let is_internal_tooling = norm_path.contains("ast-scanner/")
            || norm_path.contains("core-graph/")
            || norm_path.contains("arch-templates/")
            || norm_path.contains("ai-file-bridge/");

        let is_test_file = norm_path.contains("/test/")
            || norm_path.starts_with("test/")
            || norm_path.contains("/tests/")
            || norm_path.starts_with("tests/")
            || norm_path.contains("/testing/")
            || norm_path.starts_with("testing/")
            || norm_path.contains("/testsuite/")
            || norm_path.starts_with("testsuite/")
            || norm_path.contains("/pytest/")
            || norm_path.starts_with("pytest/")
            || norm_path.contains("integration-tests")
            || norm_path.contains("test-loop-tests")
            || norm_path.contains("test-utils")
            || norm_path.contains("test_utils")
            || norm_path.contains("testlib")
            || norm_path.contains("test-contracts")
            || norm_path.contains("runtime-params-estimator")
            || norm_path.contains("adversarial")
            || norm_path.contains("/benches/")
            || norm_path.starts_with("benches/")
            || norm_path.contains("/benchmarks/")
            || norm_path.starts_with("benchmarks/")
            || norm_path.contains("/examples/")
            || norm_path.starts_with("examples/")
            || norm_path.contains("/fixtures/")
            || norm_path.starts_with("fixtures/")
            || norm_path.contains("/testdata/")
            || norm_path.starts_with("testdata/")
            || norm_path.contains("/genesis-tools/")
            || norm_path.starts_with("genesis-tools/")
            || norm_path.contains("/devtools/")
            || norm_path.starts_with("devtools/")
            || norm_path.contains("/buildtools/")
            || norm_path.starts_with("buildtools/")
            || norm_path.contains("/tools/")
            || norm_path.starts_with("tools/")
            || norm_path.contains("debug_scripts")
            || norm_path.contains("/mock")
            || norm_path.starts_with("mock")
            || norm_path.contains("/fuzz/")
            || norm_path.starts_with("fuzz/")
            || norm_path.contains("/certora/")
            || norm_path.starts_with("certora/")
            || norm_path.contains("/script/")
            || norm_path.starts_with("script/")
            || norm_path.contains("/scripts/")
            || norm_path.starts_with("scripts/")
            || lower_file_name == "tests.rs"
            || lower_file_name == "test.rs"
            || lower_file_name == "testing.rs"
            || lower_file_name == "test_utils.rs"
            || lower_file_name == "bench.rs"
            || lower_file_name == "benches.rs"
            || lower_file_name == "build.rs"
            || lower_file_name.ends_with("_test.rs")
            || lower_file_name.ends_with("_tests.rs")
            || lower_file_name.ends_with(".pb.rs")
            || lower_file_name.ends_with(".generated.rs")
            || lower_file_name.ends_with("_generated.rs")
            || lower_file_name.starts_with("test_")
            || norm_path.ends_with(".t.sol")
            || norm_path.ends_with(".s.sol");

        // Tronquer les modules #[cfg(test)] inline AVANT d'exécuter toutes les familles de règles
        let scoped_content = if is_rust && !is_test_file {
            Self::strip_rust_inline_tests(content)
        } else {
            content
        };
        let lines: Vec<&str> = scoped_content.lines().collect();
        // Cache des recherches `content.contains("...")` : sans lui chaque ligne relance un scan
        // complet du fichier (coût quadratique, ~80 s sur de gros fichiers JS).
        let content = &ContentIndex::new(scoped_content);

        let is_interface_or_types = norm_path.contains("/interface")
            || norm_path.starts_with("interface")
            || norm_path.ends_with("interface.sol")
            || norm_path.ends_with(".d.ts")
            || (file_name.starts_with('I') && file_name.chars().nth(1).map_or(false, |c| c.is_uppercase()) && norm_path.ends_with(".sol"))
            || norm_path.contains("/types/")
            || (norm_path.ends_with(".sol") && content.contains("interface ") && !content.contains("contract "));

        // 1. Règle max_lines_per_file (Architecture)
        // Exclut formellement les tests, scripts, mocks, interfaces et déclarations de types
        if !is_test_file && !is_interface_or_types {
            if let Some(c) = contract {
                // Pour les contrats Solidity de production, relever le seuil par défaut à 600 si aucun contrat README local ne fixe de budget strict différent de 300
                let limit = if norm_path.ends_with(".sol") && c.frontmatter.max_lines_per_file == 300 {
                    600
                } else {
                    c.frontmatter.max_lines_per_file
                };

                if lines.len() > limit {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: lines.len(),
                        rule_id: "RULE_MAX_LINES".to_string(),
                        cve_id: "CWE-1075 / ARCH-BUDGET-EXCEEDED".to_string(),
                        severity: "WARNING".to_string(),
                        message: format!(
                            "File exceeds contract maximum line limit ({} > {} lines)",
                            lines.len(),
                            limit
                        ),
                        transmitted_data: Some("file_line_count".to_string()),
                        transmitted_type: Some("Integer (Total Lines of Code)".to_string()),
                        remediation: "Split logic into smaller modular sub-components or services.".to_string(),
                    });
                }
            }
        }

        // Exécution des règles avancées DeFi & EVC (Arrondis asymétriques, Cache Stale, Hook Bypass)
        violations.extend(crate::rules::AdvancedDefiRules::audit_file(
            file_path,
            content,
            is_test_file,
            is_interface_or_types,
        ));
        violations.extend(crate::deep_rules::DeepSecurityRules::audit_file(
            file_path,
            content,
            is_test_file,
            is_interface_or_types,
        ));

        let mut in_test_item = false;

        for (idx, &line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();

            // Arrêter l'audit SAST dès l'entrée dans un module unitaire #[cfg(test)] ou mod tests en fin de fichier Rust
            if is_rust
                && (trimmed == "#[cfg(test)]"
                    || trimmed.starts_with("#[cfg(all(test")
                    || trimmed == "mod tests {"
                    || trimmed == "mod test {")
            {
                break;
            }

            if is_rust {
                if trimmed == "#[test]"
                    || trimmed.starts_with("#[tokio::test")
                    || trimmed.starts_with("#[cfg(feature = \"test_features\"")
                {
                    in_test_item = true;
                } else if in_test_item
                    && (trimmed.starts_with("pub fn ")
                        || trimmed.starts_with("pub async fn ")
                        || trimmed.starts_with("pub struct ")
                        || trimmed.starts_with("impl "))
                    && !lines.iter().take(idx).rev().take(3).any(|l| {
                        let t = l.trim();
                        t == "#[test]"
                            || t.starts_with("#[tokio::test")
                            || t.starts_with("#[cfg(feature = \"test_features\"")
                    })
                {
                    in_test_item = false;
                }
                if in_test_item {
                    continue;
                }
            }

            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            if let Some(c) = contract {
                // 2. Règle forbidden_imports (Architecture & Guardrail)
                for forbidden in &c.frontmatter.forbidden_imports {
                    let clean_pat = forbidden.trim_end_matches('*').trim_end_matches(':');
                    if (trimmed.starts_with("use ") || trimmed.starts_with("import ") || trimmed.starts_with("from "))
                        && trimmed.contains(clean_pat)
                    {
                        violations.push(LinterViolation {
                            file_path: file_path.to_string(),
                            line_number: line_no,
                            rule_id: "RULE_FORBIDDEN_IMPORT".to_string(),
                            cve_id: "CWE-1075 / ARCH-CONTRACT-VIOLATION".to_string(),
                            severity: "CRITICAL".to_string(),
                            message: format!("Forbidden import detected: '{}' violates folder contract", forbidden),
                            transmitted_data: Some(forbidden.clone()),
                            transmitted_type: Some("ModuleImport (Illegal Galaxy Boundary Crossing)".to_string()),
                            remediation: format!("Remove reference to '{}'. Pass dependencies via contracts/ abstractions.", forbidden),
                        });
                    }
                }
            }

            // 3. SAST Global & Contract: no_raw_sql (CWE-89)
            let check_raw_sql = contract.map(|c| c.frontmatter.security_rules.no_raw_sql).unwrap_or(true);
            if !is_test_file
                && check_raw_sql
                && (trimmed.to_uppercase().contains("SELECT ") || trimmed.to_uppercase().contains("INSERT INTO ") || trimmed.to_uppercase().contains("UPDATE ") || trimmed.to_uppercase().contains("DELETE FROM "))
                && (trimmed.contains("${") || trimmed.contains("format!(") || trimmed.contains('+') || trimmed.contains("%s") || trimmed.contains("f\""))
            {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "SAST_RAW_SQL_INJECTION".to_string(),
                    cve_id: "CWE-89 / SQL-INJECTION".to_string(),
                    severity: "CRITICAL".to_string(),
                    message: "Dynamic SQL string concatenation detected: vulnerable to SQL Injection".to_string(),
                    transmitted_data: Some("raw_sql_query".to_string()),
                    transmitted_type: Some("String (Unescaped SQL Concatenation)".to_string()),
                    remediation: "Use parameterized queries (sqlx::query!, prepared statements, or bind parameters ? / $1).".to_string(),
                });
            }

            // 4. SAST Global & Contract: no_env_access in pure domain layer (CWE-15)
            let check_env_access = contract.map(|c| c.frontmatter.security_rules.no_env_access).unwrap_or(false)
                || lower_path.contains("domain");
            if !is_test_file
                && check_env_access
                && (trimmed.contains("std::env::var") || trimmed.contains("process.env") || trimmed.contains("os.environ"))
            {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "SAST_DIRECT_ENV_ACCESS".to_string(),
                    cve_id: "CWE-15 / DIRECT-ENV-ACCESS".to_string(),
                    severity: "HIGH".to_string(),
                    message: "Direct environment variable access in pure domain layer".to_string(),
                    transmitted_data: Some("env_var_key".to_string()),
                    transmitted_type: Some("String (Process Environment Variable)".to_string()),
                    remediation: "Pass configuration values as arguments from bootstrap / application services.".to_string(),
                });
            }

            // 5. SAST Global: Hardcoded secrets detection (CWE-798)
            if !is_test_file
                && (trimmed.contains("BEGIN PRIVATE KEY")
                    || (trimmed.contains("AKIA") && trimmed.len() > 20)
                    || ((trimmed.to_lowercase().contains("secret") || trimmed.to_lowercase().contains("api_key") || trimmed.to_lowercase().contains("private_key"))
                        && trimmed.contains(" = \"")
                        && !trimmed.contains(" = \"\"")
                        && !trimmed.contains("env")
                        && !trimmed.contains("test")
                        && !trimmed.contains("ed25519:")
                        && !trimmed.contains("alice")
                        && !trimmed.contains("bob")))
            {
                violations.push(LinterViolation {
                    file_path: file_path.to_string(),
                    line_number: line_no,
                    rule_id: "SAST_HARDCODED_SECRET".to_string(),
                    cve_id: "CWE-798 / HARDCODED-SECRET".to_string(),
                    severity: "CRITICAL".to_string(),
                    message: "Potential hardcoded credential or secret key detected in source code".to_string(),
                    transmitted_data: Some("hardcoded_credential".to_string()),
                    transmitted_type: Some("String (Plaintext Secret Key)".to_string()),
                    remediation: "Store secrets in secure vault / environment variables, never commit secrets to code.".to_string(),
                });
            }

            // =========================================================
            // 🦀 RÈGLES SPÉCIFIQUES RUST & RÉSEAU BAS-NIVEAU + BLOCKCHAIN L1
            // =========================================================
            if is_rust && !is_test_file {
                // --- RÈGLES L1 BLOCKCHAIN NODE ---
                // L1.1 CVE-L1-BORSH-UNBOUNDED-DESER-OOM (Borsh / Zstd Unbounded Network or StateWitness Deserialization)
                if !trimmed.starts_with("use ")
                    && !trimmed.starts_with("pub fn try_from_slice")
                    && !trimmed.starts_with("fn try_from_slice")
                    && !norm_path.contains("proto_conv/")
                    && !norm_path.contains("rosetta-rpc/")
                    && !norm_path.contains("jsonrpc/src/api/")
                    && ((trimmed.contains("PeerMessage::deserialize(")
                        && content.contains("peer_msg.max_size()"))
                        || ((trimmed.contains("BorshDeserialize::try_from_slice(")
                            || ((trimmed.contains("::try_from_slice(")
                                || trimmed.contains("BorshDeserialize::deserialize("))
                                && (trimmed.contains("raw_bytes")
                                    || trimmed.contains("raw_msg")
                                    || trimmed.contains("network_msg")
                                    || trimmed.contains("witness_bytes")
                                    || trimmed.contains("payload")))
                            || trimmed.contains("zstd::decode_all(")
                            || trimmed.contains("zstd::stream::copy_decode("))
                            && (lower_path.contains("network")
                                || lower_path.contains("witness")
                                || lower_path.contains("chunk")
                                || lower_path.contains("client")
                                || lower_path.contains("peer")
                                || lower_path.contains("rpc")
                                || trimmed.contains("raw_bytes")
                                || trimmed.contains("raw_msg")
                                || trimmed.contains("network_msg"))
                            && !content.contains("MAX_")
                            && !content.contains("max_size")
                            && !content.contains("max_len")
                            && !content.contains("with_limit")
                            && !content.contains("size_limit")
                            && !content.contains(".take(")))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-L1-BORSH-UNBOUNDED-DESER-OOM".to_string(),
                        cve_id: "L1-P2P-CONSENSUS-CRITICAL / BORSH-ZSTD-WIRE-ALLOCATION-BOMB".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Borsh deserialization or zstd decompression on P2P network/witness stream without size cap or .take(MAX_LIMIT): malicious peer can send a forged 4GB length prefix or zstd bomb to OOM-crash validator nodes".to_string(),
                        transmitted_data: Some("p2p_borsh_or_zstd_payload".to_string()),
                        transmitted_type: Some("&[u8] (Uncapped Wire Borsh / Zstd Stream)".to_string()),
                        remediation: "Enforce a strict byte ceiling (.take(MAX_DECOMPRESSED_BYTES) / size_limit) and verify validator signature BEFORE Borsh deserialization or zstd decompression.".to_string(),
                    });
                }

                // L1.2 CVE-L1-RUNTIME-GAS-PREPAY-BYPASS (Wasm Host Function Allocation / I/O Before burn_gas)
                let alloc_pos = trimmed
                    .find("vec![0u8;")
                    .or_else(|| trimmed.find("vec![0;"))
                    .or_else(|| trimmed.find("read_memory("))
                    .or_else(|| trimmed.find("Vec::with_capacity("));
                let gas_paid_before_on_same_line = trimmed
                    .find("burn_gas")
                    .or_else(|| trimmed.find("pay_gas"))
                    .or_else(|| trimmed.find("prepay_gas"))
                    .map_or(false, |g_pos| alloc_pos.map_or(false, |a_pos| g_pos < a_pos));
                let is_wasm_host_logic = (norm_path.contains("vm-runner") || norm_path.contains("logic") || norm_path.contains("host"))
                    && !norm_path.contains("compiler_daemon")
                    && !norm_path.contains("prepare")
                    && !norm_path.contains("bandwidth_scheduler")
                    && !norm_path.contains("prefetch")
                    && (content.contains("gas_counter") || content.contains("burn_gas") || content.contains("pay_gas"));
                if is_wasm_host_logic
                    && (trimmed.contains("read_memory(")
                        || trimmed.contains("vec![0u8;")
                        || trimmed.contains("vec![0;")
                        || (trimmed.contains("Vec::with_capacity(") && trimmed.contains("len")))
                    && !gas_paid_before_on_same_line
                    && !lines.iter().take(idx).rev().take(12).any(|l| {
                        l.contains("burn_gas")
                            || l.contains("pay_gas")
                            || l.contains("deduct_gas")
                            || l.contains("gas_counter")
                            || l.contains("prepay_gas")
                            || l.contains("MAX_")
                            || l.contains("max_len")
                    })
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-L1-RUNTIME-GAS-PREPAY-BYPASS".to_string(),
                        cve_id: "L1-FEE-BYPASS-CRITICAL / HOST-FN-ALLOCATION-BEFORE-GAS-BURN".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Wasm runtime host function allocates host buffer or reads guest memory before charging proportional gas (burn_gas / pay_gas): allows a malicious contract to trigger free multi-GB host allocations before reverting OutOfGas".to_string(),
                        transmitted_data: Some("unbilled_wasm_host_allocation".to_string()),
                        transmitted_type: Some("u64 -> Vec<u8> (Host Memory Allocated Before Gas Burn)".to_string()),
                        remediation: "Charge proportional gas via gas_counter.burn_gas(len * byte_fee) and validate len <= max_memory_read_size BEFORE allocating host memory.".to_string(),
                    });
                }

                // L1.3 CVE-L1-BALANCE-STORAGE-STAKING-DESYNC (Balance / Stake Mutation Without Storage Stake Check or Checked Math)
                if (lower_path.contains("runtime") || lower_path.contains("balance") || lower_path.contains("stake"))
                    && !norm_path.contains("primitives")
                    && !trimmed.contains("fn set_amount")
                    && !trimmed.contains("fn set_locked")
                    && ((trimmed.contains(".set_amount(") || trimmed.contains(".set_locked("))
                        && (trimmed.contains("saturating_sub(") || trimmed.contains(" - ") || trimmed.contains(" + "))
                        || (trimmed.contains("account.set_amount(")
                            && !content.contains("check_storage_stake")
                            && !content.contains("safe_add_balance")
                            && !content.contains("checked_sub")
                            && !content.contains("checked_add")))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-L1-BALANCE-STORAGE-STAKING-DESYNC".to_string(),
                        cve_id: "L1-FUNDS-CRITICAL / BALANCE-MUTATION-WITHOUT-STORAGE-STAKE-CHECK".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "L1 account balance or locked stake mutated via raw arithmetic/saturating_sub or without check_storage_stake invariant verification: risks storage staking bypass or balance inflation".to_string(),
                        transmitted_data: Some("account_balance_or_stake_delta".to_string()),
                        transmitted_type: Some("Balance / u128 (Unverified Storage Stake Invariant)".to_string()),
                        remediation: "Use safe_add_balance / checked_sub returning RuntimeError::BalanceOverflow and invoke check_storage_stake(&account, &runtime_config) after state mutation.".to_string(),
                    });
                }

                // L1.4 CVE-L1-CONSENSUS-UNVERIFIED-WITNESS-CHUNK (Stateless Validation Witness / Endorsement Processed Before Signature Check)
                if !trimmed.starts_with("use ")
                    && (trimmed.contains("fn handle_chunk_state_witness")
                        || trimmed.contains("fn process_chunk_state_witness")
                        || trimmed.contains("fn on_chunk_state_witness"))
                    && (content.contains("apply_transactions")
                        || content.contains("apply_chunk_state_witness")
                        || content.contains("save_state_witness")
                        || content.contains("save_latest_witness"))
                    && !content.contains("verify_chunk_state_witness")
                    && !content.contains("verify_signature")
                    && !content.contains("validate_chunk_endorsement")
                    && !content.contains("validator_signer")
                    && !content.contains("decode_and_validate")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-L1-CONSENSUS-UNVERIFIED-WITNESS-CHUNK".to_string(),
                        cve_id: "L1-CONSENSUS-CRITICAL / STATELESS-WITNESS-UNVERIFIED-ENDORSEMENT".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Stateless validation ChunkStateWitness processed or persisted before verifying chunk producer cryptographic signature: Byzantine node can poison witness cache or stall shard validation".to_string(),
                        transmitted_data: Some("unverified_chunk_state_witness".to_string()),
                        transmitted_type: Some("ChunkStateWitness (Unauthenticated Producer State Transition)".to_string()),
                        remediation: "Verify the chunk producer's cryptographic signature via epoch_manager.verify_chunk_state_witness_signature(&witness) BEFORE decoding or applying the witness.".to_string(),
                    });
                }

                // L1.5 CVE-L1-CROSS-SHARD-RECEIPT-NONCE-REPLAY (NEP-366 DelegateAction / AccessKey Nonce & Allowance Bypass)
                if !trimmed.starts_with("use ")
                    && !norm_path.contains("primitives")
                    && !norm_path.contains("rosetta-rpc")
                    && !norm_path.contains("prefetch")
                    && (trimmed.contains("fn verify_delegate_action")
                        || trimmed.contains("fn validate_delegate_action")
                        || trimmed.contains("fn verify_and_charge_tx")
                        || trimmed.contains("fn process_delegate_action"))
                    && (content.contains("ActionReceipt") || content.contains("access_key") || content.contains("delegate_action"))
                    && !content.contains("max_block_height")
                    && !content.contains("access_key.nonce")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-L1-CROSS-SHARD-RECEIPT-NONCE-REPLAY".to_string(),
                        cve_id: "L1-TX-CRITICAL / DELEGATE-ACTION-ACCESS-KEY-NONCE-BYPASS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "NEP-366 DelegateAction or transaction receipt processed without enforcing block_height <= max_block_height and atomic access_key.nonce increment: enables cross-shard meta-transaction replay".to_string(),
                        transmitted_data: Some("delegate_action_receipt".to_string()),
                        transmitted_type: Some("SignedDelegateAction (Unchecked Nonce / Block Height Expiry)".to_string()),
                        remediation: "Enforce block_height <= delegate_action.max_block_height, reject nested DelegateActions, and atomically update access_key.nonce before emitting ActionReceipt.".to_string(),
                    });
                }

                // L1.6 CVE-L1-CRYPTO-BLIND-SCALAR-MERKLE-MALLEABILITY (Unchecked Elliptic Curve Decompression / Small-Subgroup Consensus Split)
                if (trimmed.contains("from_bytes_unchecked(")
                    || trimmed.contains("decompress_unchecked(")
                    || trimmed.contains("from_ compressed_unchecked(")
                    || (trimmed.contains("ed25519_dalek") && trimmed.contains("verify(") && !content.contains("verify_strict")))
                    && !content.contains("is_torsion_free")
                    && !content.contains("is_small_order")
                    && !content.contains(" subgroup_check")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-L1-CRYPTO-BLIND-SCALAR-MERKLE-MALLEABILITY".to_string(),
                        cve_id: "L1-CRYPTO-CRITICAL / UNCHECKED-CURVE-DECOMPRESS-TORSION-SPLIT".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Cryptographic curve point decompressed via *_unchecked() or non-strict Ed25519 verify without prime-order subgroup check (is_torsion_free / verify_strict): risks consensus split across validators".to_string(),
                        transmitted_data: Some("unchecked_curve_point_or_sig".to_string()),
                        transmitted_type: Some("CurvePoint / Signature (Small-Subgroup Torsion Malleability)".to_string()),
                        remediation: "Use canonical decompression with .is_torsion_free() on BLS12-381/BN254 points and verify_strict() on Ed25519 signatures.".to_string(),
                    });
                }

                // 0.1 CVE-RUST-DESER-BOMB-OOM (CWE-789 / Length-Prefix Allocation Bomb)
                if (trimmed.contains("bincode::deserialize(")
                    || trimmed.contains("bincode::deserialize_from(")
                    || trimmed.contains("rmp_serde::from_slice(")
                    || trimmed.contains("rmp_serde::from_read(")
                    || (trimmed.contains("Vec::with_capacity(")
                        && (trimmed.contains("packet") || trimmed.contains("wire") || trimmed.contains("frame") || trimmed.contains("client") || trimmed.contains("raw_len") || trimmed.contains("claimed_len"))))
                    && !content.contains("with_limit(")
                    && !trimmed.contains(".min(")
                    && !content.contains("MAX_FRAME_SIZE")
                    && !content.contains("MAX_PACKET_BYTES")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-DESER-BOMB-OOM".to_string(),
                        cve_id: "CWE-789 / LENGTH-PREFIX-OOM-BOMB".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Unbounded length-prefix deserialization or pre-allocation: an 8-byte forged header triggers immediate Rust allocator OOM abort()".to_string(),
                        transmitted_data: Some("wire_frame_length_prefix".to_string()),
                        transmitted_type: Some("u64 / &[u8] (Uncapped Pre-Allocation Prefix)".to_string()),
                        remediation: "Use bincode::DefaultOptions::new().with_limit(MAX_PACKET_BYTES).deserialize(bytes) or cap capacity with len.min(MAX_FRAME_SIZE).".to_string(),
                    });
                }

                // 0.2 CVE-RUST-CANCEL-SAFETY-LOSS (CWE-662 / Tokio Select! Non-Cancel-Safe I/O)
                if (content.contains("tokio::select!") || content.contains("select! {"))
                    && (trimmed.contains(".read_exact(")
                        || trimmed.contains(".write_all(")
                        || trimmed.contains(".read_to_end(")
                        || trimmed.contains(".read_line("))
                    && !content.contains("Framed::new")
                    && !content.contains("cancel_safe")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-CANCEL-SAFETY-LOSS".to_string(),
                        cve_id: "CWE-662 / TOKIO-SELECT-CANCEL-LOSS".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Non-cancel-safe async I/O (read_exact/write_all/read_line) inside tokio::select! silently drops partial TCP frame bytes on branch cancellation".to_string(),
                        transmitted_data: Some("partial_tcp_frame_buffer".to_string()),
                        transmitted_type: Some("Future (Non-Cancel-Safe AsyncRead/AsyncWrite)".to_string()),
                        remediation: "Use tokio_util::codec::Framed with StreamExt::next() (cancel-safe stateful BytesMut) or move multi-step I/O outside tokio::select!.".to_string(),
                    });
                }

                // 0.3 CVE-RUST-TIMING-SIDECHANNEL (CWE-208 / Non-Constant-Time Secret Compare)
                if (trimmed.contains(" == ") || trimmed.contains(" != "))
                    && (trimmed.contains("expected_hmac")
                        || trimmed.contains("hmac")
                        || trimmed.contains("secret_token")
                        || trimmed.contains("mac_tag")
                        || trimmed.contains("auth_digest")
                        || ((trimmed.contains("expected") || trimmed.contains("signature") || trimmed.contains("secret") || trimmed.contains("stored"))
                            && (lower_path.contains("auth") || lower_path.contains("cache") || lower_path.contains("session") || lower_path.contains("token"))))
                    && !trimmed.contains("None")
                    && !trimmed.contains("0")
                    && !trimmed.contains("\"\"")
                    && !content.contains("ct_eq(")
                    && !content.contains("ConstantTimeEq")
                    && !content.contains("constant_time")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-TIMING-SIDECHANNEL".to_string(),
                        cve_id: "CWE-208 / TIMING-SIDE-CHANNEL".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Non-constant-time comparison (== / !=) on cryptographic secret, nonce, or HMAC allows remote byte-by-byte timing side-channel recovery".to_string(),
                        transmitted_data: Some("secret_or_nonce_digest".to_string()),
                        transmitted_type: Some("&str / &[u8] (Short-Circuit SIMD memcmp)".to_string()),
                        remediation: "Use constant-time comparison via subtle::ConstantTimeEq: bool::from(stored.as_bytes().ct_eq(expected.as_bytes())).".to_string(),
                    });
                }

                // 0.4 CVE-NET-SLOWLORIS-KEEPALIVE-DOS (CWE-400 / Unbounded Socket Handshake & Read)
                if (trimmed.contains("accept_async(")
                    || trimmed.contains("tokio_tungstenite::accept")
                    || trimmed.contains("listener.accept()")
                    || trimmed.contains("axum::serve(")
                    || (trimmed.contains("acquire_many_owned(")
                        && content.contains("NETWORK_MESSAGE_MAX_SIZE_BYTES")
                        && content.contains("read_exact(")))
                    && !content.contains("TimeoutLayer")
                    && (!content.contains("tokio::time::timeout") || trimmed.contains("acquire_many_owned("))
                    && (!content.contains("timeout(") || trimmed.contains("acquire_many_owned("))
                    && !lower_path.contains("desktop-shell")
                    && !norm_path.contains("tracing/")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-NET-SLOWLORIS-KEEPALIVE-DOS".to_string(),
                        cve_id: "CWE-400 / SLOWLORIS-SOCKET-EXHAUSTION".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Network socket, HTTP server, or P2P global semaphore permit acquired (up to 512 MiB) before payload bytes arrive: vulnerable to Slowloris global semaphore starvation & fd exhaustion".to_string(),
                        transmitted_data: Some("tcp_handshake_or_p2p_semaphore_permit".to_string()),
                        transmitted_type: Some("TcpStream / OwnedSemaphorePermit (Slowloris Resource Starvation)".to_string()),
                        remediation: "Apply tower_http::timeout::TimeoutLayer on HTTP servers, cap pre-handshake frame sizes to MAX_SMALL_MESSAGE_SIZE, and isolate per-class connection semaphores.".to_string(),
                    });
                }

                // 0.5 CVE-NET-SSRF-DNS-REBIND (CWE-918 / Outbound SSRF & DNS Rebinding TOCTOU)
                let is_tainted_ssrf_target = trimmed.contains("webhook")
                    || trimmed.contains("callback_url")
                    || trimmed.contains("user_url")
                    || trimmed.contains("target_url")
                    || trimmed.contains("remote_url")
                    || ((norm_path.contains("webhook") || norm_path.contains("proxy") || norm_path.contains("auth"))
                        && (trimmed.contains("url") || trimmed.contains("uri")));
                if (trimmed.contains("reqwest::get(")
                    || ((trimmed.contains("client.get(") || trimmed.contains("client.post("))
                        && !trimmed.contains("gcs_client.")
                        && !trimmed.contains("s3_client.")
                        && !trimmed.contains("rpc_client.")
                        && is_tainted_ssrf_target)
                    || (trimmed.contains("TcpStream::connect(") && is_tainted_ssrf_target))
                    && !norm_path.contains("download_file")
                    && !norm_path.contains("external-storage")
                    && !norm_path.contains("jsonrpc/client")
                    && !trimmed.contains("\"http")
                    && !trimmed.contains("\"127.0.0.1")
                    && !content.contains("is_loopback()")
                    && !content.contains("is_private()")
                    && !content.contains("Policy::none()")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-NET-SSRF-DNS-REBIND".to_string(),
                        cve_id: "CWE-918 / SSRF-DNS-REBINDING".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Outbound network connection to dynamic target without post-DNS IP validation (is_private/is_loopback/169.254.169.254): vulnerable to SSRF & DNS Rebinding".to_string(),
                        transmitted_data: Some("remote_webhook_or_uri".to_string()),
                        transmitted_type: Some("Url / SocketAddr (Unpinned DNS Resolution)".to_string()),
                        remediation: "Enforce redirect::Policy::none() and resolve DNS once, rejecting any IpAddr where ip.is_loopback() || ip.is_private() || ip.is_link_local().".to_string(),
                    });
                }

                // CVE-RUST-TRANSMUTE-SIZE-MISMATCH (CWE-843 / Unsafe Transmute Type Confusion)
                let window_start = idx.saturating_sub(6);
                let window_end = (idx + 4).min(lines.len());
                let has_safety_comment = lines[window_start..window_end].iter().any(|l| {
                    let up = l.to_uppercase();
                    up.contains("// SAFETY:")
                        || up.contains("// SAFE:")
                        || up.contains("/* SAFETY:")
                        || l.contains("is_torsion_free()")
                        || l.contains("repr(transparent)")
                });
                let is_benign_unsafe = trimmed.contains("unreachable_unchecked")
                    || trimmed.contains("from_utf8_unchecked")
                    || trimmed.contains("env::set_var")
                    || trimmed.contains("env::remove_var")
                    || trimmed.contains("blst::")
                    || lines[idx..(idx + 4).min(lines.len())]
                        .iter()
                        .any(|l| l.contains("env::set_var") || l.contains("env::remove_var") || l.contains("unreachable_unchecked") || l.contains("blst::"))
                    || trimmed.contains("PhantomData<*const")
                    || trimmed.contains("PhantomData<*mut")
                    || trimmed.contains("unsafe trait ")
                    || trimmed.contains("unsafe impl ")
                    || trimmed.contains("unsafe fn ")
                    || norm_path.contains("bls12381.rs")
                    || norm_path.contains("core/store/src/db/rocksdb")
                    || norm_path.contains("core/store/src/db/slice")
                    || norm_path.contains("core/store/src/trie/mem");

                if !trimmed.starts_with("use ")
                    && (trimmed.contains("transmute(") || trimmed.contains("transmute::<"))
                    && (trimmed.contains("mem::") || trimmed.contains("std::mem") || content.contains("mem::transmute"))
                    && !content.contains("bytemuck")
                    && !has_safety_comment
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-TRANSMUTE-SIZE-MISMATCH".to_string(),
                        cve_id: "CWE-843 / UNSAFE-TRANSMUTE-TYPE-CONFUSION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Arbitrary type transmutation (std::mem::transmute): undefined behavior if layouts differ".to_string(),
                        transmitted_data: Some("transmute_target".to_string()),
                        transmitted_type: Some("transmute<T, U> (Arbitrary Memory Reinterpretation)".to_string()),
                        remediation: "Use safe conversions (From/Into, TryFrom, or safe bytemuck casts).".to_string(),
                    });
                } else if (trimmed.contains("unsafe ") || trimmed.starts_with("unsafe{") || trimmed == "unsafe")
                    && !has_safety_comment
                    && !is_benign_unsafe
                    && (trimmed.contains("packet_ptr")
                        || trimmed.contains("raw_ptr")
                        || trimmed.contains("*ptr")
                        || trimmed.contains("copy_nonoverlapping")
                        || trimmed.contains("from_raw_parts")
                        || trimmed.contains("get_unchecked(")
                        || is_web3_mmo
                        || norm_path.contains("server")
                        || norm_path.contains("network")
                        || norm_path.contains("rpc")
                        || norm_path.contains("runtime"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-UNSAFE-MEM".to_string(),
                        cve_id: "CWE-119 / CVE-RUST-UNSAFE-MEM".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Undocumented unsafe block (missing // SAFETY: invariant justification): bypasses Rust borrow-checker and memory safety invariants".to_string(),
                        transmitted_data: Some("unsafe_memory_block".to_string()),
                        transmitted_type: Some("UnsafeBlock (Raw Memory Invariants Bypassed)".to_string()),
                        remediation: "Avoid unsafe blocks; utilize safe Rust references (&T, &mut T), Box, NonNull, or document proved invariants with // SAFETY:.".to_string(),
                    });
                } else if (trimmed.contains("*const ") || trimmed.contains("*mut "))
                    && !has_safety_comment
                    && !is_benign_unsafe
                    && !trimmed.contains("extern \"C\"")
                    && !trimmed.starts_with("pub fn ")
                    && !trimmed.starts_with("fn ")
                    && (trimmed.contains("packet_ptr") || trimmed.contains("raw_ptr") || is_web3_mmo)
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-UNSAFE-MEM".to_string(),
                        cve_id: "CWE-119 / CVE-RUST-UNSAFE-MEM".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Raw pointer dereference or declaration: risk of null pointer dereference, UAF or buffer overflow".to_string(),
                        transmitted_data: Some("*mut packet_ptr".to_string()),
                        transmitted_type: Some("*mut T / *const T (Raw Pointer)".to_string()),
                        remediation: "Use safe references (&T, &mut T) or Box/NonNull.".to_string(),
                    });
                }

                // CVE-RUST-PANIC-DOS (CWE-248 / Denial of Service)
                if (lower_path.contains("server") || lower_path.contains("api") || lower_path.contains("domain"))
                    && !norm_path.contains("openapi/")
                    && (trimmed.contains(".unwrap()") || trimmed.contains(".expect("))
                    && !trimmed.contains("panic =")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-PANIC-DOS".to_string(),
                        cve_id: "CWE-248 / CVE-RUST-PANIC-DOS".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Direct .unwrap() or .expect() on network/user inputs in server layer: malicious payload can crash process".to_string(),
                        transmitted_data: Some("unverified_payload".to_string()),
                        transmitted_type: Some("Option/Result (Unchecked Panic on None/Err)".to_string()),
                        remediation: "Handle errors gracefully using '?' operator, match/if let, or return HTTP 400.".to_string(),
                    });
                }

                // CVE-RUST-CMD-INJ (CWE-78 / OS Command Injection)
                if (trimmed.contains("Command::new(") || trimmed.contains("process::Command::new("))
                    && (trimmed.contains("\"sh\"")
                        || trimmed.contains("\"bash\"")
                        || trimmed.contains("\"cmd\"")
                        || trimmed.contains("\"powershell\"")
                        || (!trimmed.contains("Command::new(\"")
                            && !trimmed.contains("current_exe")
                            && !trimmed.contains("Command::new(binary)")
                            && !trimmed.contains("cargo")
                            && !trimmed.contains("self.")
                            && !norm_path.contains("cli.rs")
                            && !norm_path.contains("compiler_daemon")))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-CMD-INJ".to_string(),
                        cve_id: "CWE-78 / CVE-RUST-CMD-INJ".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "OS Command execution via Command::new: risk of arbitrary OS command injection".to_string(),
                        transmitted_data: Some("cmd_arguments".to_string()),
                        transmitted_type: Some("String (Dynamic Shell Argument)".to_string()),
                        remediation: "Validate parameters against an allowlist and do not invoke shell interpreters directly.".to_string(),
                    });
                }

                // CVE-RUST-ASYNC-MUTEX (CWE-667 / Async Deadlock)
                if trimmed.contains("std::sync::Mutex") && content.contains(".await") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-ASYNC-MUTEX".to_string(),
                        cve_id: "CWE-667 / ASYNC-DEADLOCK".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "std::sync::Mutex held across await points blocks the async threadpool runtime".to_string(),
                        transmitted_data: Some("std_sync_mutex".to_string()),
                        transmitted_type: Some("std::sync::MutexGuard across .await".to_string()),
                        remediation: "Use tokio::sync::Mutex or restructure logic so the guard is dropped before .await.".to_string(),
                    });
                }

                // CVE-RUST-UNBOUNDED-CHANNEL (CWE-400 / Memory DoS)
                if (lower_path.contains("server") || lower_path.contains("api") || lower_path.contains("websocket"))
                    && trimmed.contains("unbounded_channel(")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-UNBOUNDED-CHANNEL".to_string(),
                        cve_id: "CWE-400 / UNBOUNDED-MEMORY-DOS".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Unbounded channel used for network packet buffering: malicious client can exhaust server RAM".to_string(),
                        transmitted_data: Some("unbounded_channel".to_string()),
                        transmitted_type: Some("tokio::sync::mpsc::UnboundedReceiver (Uncapped Queue)".to_string()),
                        remediation: "Use bounded tokio::sync::mpsc::channel(CAPACITY) with backpressure.".to_string(),
                    });
                }

                // CVE-MATH-UNCHECKED-BALANCE (CWE-190 / Integer Underflow)
                if is_web3_mmo
                    && (trimmed.contains("balance -=") || trimmed.contains("coins -=") || trimmed.contains("gold -=") || trimmed.contains("tokens -=") || trimmed.contains("pixel_balance -="))
                    && !trimmed.contains("saturating_sub")
                    && !trimmed.contains("checked_sub")
                    && !content.contains("checked_sub")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-MATH-UNCHECKED-BALANCE".to_string(),
                        cve_id: "CWE-190 / INTEGER-UNDERFLOW".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Direct balance deduction without checked_sub or saturating_sub risks integer underflow wrapping".to_string(),
                        transmitted_data: Some("balance_decrement".to_string()),
                        transmitted_type: Some("u64 / u128 (Unchecked Underflow)".to_string()),
                        remediation: "Use balance.checked_sub(amount).ok_or(...) or saturating_sub(amount).".to_string(),
                    });
                }

                // CVE-MMO-UNCHECKED-SPEEDHACK (CWE-20 / Speedhack)
                if (lower_path.contains("movement") || lower_path.contains("world") || lower_path.contains("player"))
                    && (trimmed.contains("x = packet.x")
                        || trimmed.contains("y = packet.y")
                        || trimmed.contains("player.x =")
                        || trimmed.contains("entity.x =")
                        || trimmed.contains("current + delta"))
                    && !content.contains("max_speed")
                    && !content.contains("clamp(")
                    && !content.contains("validate_move")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-MMO-UNCHECKED-SPEEDHACK".to_string(),
                        cve_id: "CWE-20 / UNCHECKED-SPEEDHACK".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Player position updated directly from packet/delta without server-side max_speed or clamp validation".to_string(),
                        transmitted_data: Some("player_coordinates".to_string()),
                        transmitted_type: Some("f32 / f64 (Unvalidated Position Coords)".to_string()),
                        remediation: "Clamp movement delta to [-MAX_SPEED * dt, MAX_SPEED * dt] and validate collision bounds before updating coordinates.".to_string(),
                    });
                }

                // 0.6 CVE-RUST-BLOCKING-IN-ASYNC (CWE-400 / Async Executor Starvation)
                let in_async_fn_scope = {
                    let mut is_async = false;
                    for prev in lines.iter().take(idx + 1).rev().take(30) {
                        let pt = prev.trim();
                        if pt.contains("async fn ") || pt.contains("async move") {
                            is_async = true;
                            break;
                        }
                        if pt.starts_with("fn ") || pt.starts_with("pub fn ") || pt.starts_with("pub(crate) fn ") {
                            is_async = false;
                            break;
                        }
                    }
                    is_async
                };
                if in_async_fn_scope
                    && (trimmed.contains("std::fs::")
                        || trimmed.contains("std::thread::sleep(")
                        || trimmed.contains("thread::sleep(")
                        || trimmed.contains("std::net::TcpStream"))
                    && !content.contains("spawn_blocking")
                    && !content.contains("block_in_place")
                    && !norm_path.contains("cli.rs")
                    && !norm_path.contains("subscriber.rs")
                    && !norm_path.contains("external-storage")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-BLOCKING-IN-ASYNC".to_string(),
                        cve_id: "CWE-400 / ASYNC-EXECUTOR-STARVATION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Synchronous blocking syscall (std::fs / thread::sleep / std::net) executed inside async context without spawn_blocking: starves Tokio worker threads".to_string(),
                        transmitted_data: Some("blocking_syscall_in_async".to_string()),
                        transmitted_type: Some("OS Thread Block (Tokio Executor Starvation)".to_string()),
                        remediation: "Replace with tokio::fs / tokio::time::sleep or offload blocking work via tokio::task::spawn_blocking(move || ...).await.".to_string(),
                    });
                }

                // CVE-RUST-INTEGER-OVERFLOW-CAST (CWE-190 / As-Cast Truncation)
                let has_prior_max_check = lines.iter().take(idx + 1).rev().take(8).any(|l| {
                    l.contains("MAX_")
                        || l.contains("INLINED_MASK")
                        || l.contains("saturating_add")
                        || l.contains("assert!")
                });
                if (trimmed.contains(" as u32") || trimmed.contains(" as u16") || trimmed.contains(" as u8"))
                    && (trimmed.contains("len") || trimmed.contains("balance") || trimmed.contains("amount") || trimmed.contains("size") || trimmed.contains("cap") || trimmed.contains("packet"))
                    && !trimmed.contains("epoch_length")
                    && !trimmed.contains("try_into()")
                    && !trimmed.contains("try_from(")
                    && !trimmed.contains(".min(")
                    && !trimmed.contains("u32::from")
                    && !trimmed.contains("u16::from")
                    && !trimmed.contains("u8::from")
                    && !has_prior_max_check
                    && !is_internal_tooling
                    && !norm_path.contains("core/store/src/trie")
                    && !norm_path.contains("core/primitives/src/state.rs")
                    && !norm_path.contains("core/async/src/instrumentation")
                    && !norm_path.contains("raw/connection.rs")
                    && !lower_path.contains("test")
                    && !lower_path.contains("desktop-shell")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-INTEGER-OVERFLOW-CAST".to_string(),
                        cve_id: "CWE-190 / AS-CAST-INTEGER-TRUNCATION".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Silent integer truncation via 'as' cast on length/size/balance operand without bounds check or TryFrom/TryInto".to_string(),
                        transmitted_data: Some("truncated_integer_operand".to_string()),
                        transmitted_type: Some("u64 / usize (Silent Truncating 'as' Cast)".to_string()),
                        remediation: "Use u32::try_from(val).map_err(...) or val.try_into() with explicit error handling.".to_string(),
                    });
                }

                // CVE-RUST-FORGOTTEN-MEM-LEAK (CWE-775 / Memory & Deadlock Resource Leak via mem::forget)
                if (trimmed.contains("mem::forget(") || trimmed.contains("std::mem::forget("))
                    && (trimmed.contains("guard") || trimmed.contains("lock") || trimmed.contains("mutex") || trimmed.contains("socket") || trimmed.contains("handle"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-FORGOTTEN-MEM-LEAK".to_string(),
                        cve_id: "CWE-775 / MEM-FORGET-RESOURCE-LEAK-DEADLOCK".to_string(),
                        severity: "HIGH".to_string(),
                        message: "std::mem::forget called on synchronization guard or I/O handle: bypasses RAII Drop cleanup and causes permanent deadlock or descriptor leak".to_string(),
                        transmitted_data: Some("leaked_raii_guard".to_string()),
                        transmitted_type: Some("MutexGuard / Resource Handle (Bypassed Drop Destructor)".to_string()),
                        remediation: "Let RAII Drop run naturally or use explicit drop(guard) instead of mem::forget.".to_string(),
                    });
                }

                // CVE-RUST-UNCHECKED-INDEX-SLICING (CWE-125 / Unchecked Slice Index Panic DOS)
                if (trimmed.contains("packet[") || trimmed.contains("wire_bytes[") || trimmed.contains("frame[") || trimmed.contains("payload[") || trimmed.contains("raw_buf["))
                    && (trimmed.contains("..") || trimmed.contains("+"))
                    && !trimmed.contains(".get(")
                    && !trimmed.contains("if ")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-UNCHECKED-INDEX-SLICING".to_string(),
                        cve_id: "CWE-125 / UNCHECKED-SLICE-INDEX-PANIC-DOS".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Direct slice indexing [start..end] on raw network packet buffer without .get(..): malformed packet triggers thread panic DoS".to_string(),
                        transmitted_data: Some("raw_slice_bounds".to_string()),
                        transmitted_type: Some("&[u8] (Direct Panic-Prone Range Indexing)".to_string()),
                        remediation: "Use packet.get(offset..end).ok_or(PacketError::Truncated) to handle bounds safely.".to_string(),
                    });
                }

                // CVE-RUST-THREAD-POISON-IGNORED (CWE-391 / Swallowed Mutex PoisonError)
                if (trimmed.contains(".lock().unwrap_or_else(") || trimmed.contains(".read().unwrap_or_else(") || trimmed.contains(".write().unwrap_or_else("))
                    && (trimmed.contains("into_inner()") || trimmed.contains("PoisonError"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-THREAD-POISON-IGNORED".to_string(),
                        cve_id: "CWE-391 / SWALLOWED-MUTEX-POISON-INVARIANT".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Swallowing Mutex PoisonError via into_inner() without repairing corrupted shared state invariants after a thread panic".to_string(),
                        transmitted_data: Some("poisoned_mutex_guard".to_string()),
                        transmitted_type: Some("PoisonError<MutexGuard> (Compromised Invariants)".to_string()),
                        remediation: "Treat PoisonError as fatal or re-validate and repair corrupted data invariants before proceeding.".to_string(),
                    });
                }

                // CVE-RUST-FFI-UNCHECKED-NULL-PTR (CWE-476 / FFI Unchecked Null Pointer Dereference)
                if (trimmed.contains("unsafe { &*") || trimmed.contains("unsafe { &mut *") || trimmed.contains("&*ptr") || trimmed.contains("&mut *ptr") || trimmed.contains("unsafe { *ptr") || trimmed.contains("*ptr"))
                    && !has_safety_comment
                    && !is_benign_unsafe
                    && !trimmed.contains("NonNull")
                    && !content.contains("is_null()")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-FFI-UNCHECKED-NULL-PTR".to_string(),
                        cve_id: "CWE-476 / FFI-UNCHECKED-NULL-PTR-DEREF".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Raw pointer dereference in unsafe block without prior is_null() check: risks null pointer dereference SIGSEGV".to_string(),
                        transmitted_data: Some("raw_ffi_pointer".to_string()),
                        transmitted_type: Some("*mut T / *const T (Unchecked Raw Null Pointer)".to_string()),
                        remediation: "Verify with ptr.is_null() or wrap via NonNull::new(ptr).ok_or(...) before dereferencing.".to_string(),
                    });
                }

                // CVE-RUST-PIN-STRUCTURAL-UNPIN-SOUNDNESS-UB (RUST-INTERNALS-CRITICAL / Pin Soundness Break)
                if (trimmed.contains("Pin::get_unchecked_mut(")
                    || trimmed.contains("Pin::new_unchecked(")
                    || trimmed.contains(".get_unchecked_mut()"))
                    && !content.contains("pin_project!")
                    && !content.contains("#[pin_project]")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-RUST-PIN-STRUCTURAL-UNPIN-SOUNDNESS-UB".to_string(),
                        cve_id: "RUST-INTERNALS-CRITICAL / PIN-GET-UNCHECKED-MUT-SOUNDNESS-BREAK".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Manual unsafe Pin projection (Pin::get_unchecked_mut / Pin::new_unchecked) without pin_project!: moving or swapping a self-referential Future across .await points causes silent Use-After-Free Undefined Behavior".to_string(),
                        transmitted_data: Some("pinned_self_referential_future".to_string()),
                        transmitted_type: Some("Pin<&mut Self> (Unverified Structural Pinning Invariant)".to_string()),
                        remediation: "Use the #[pin_project] macro from pin-project-lite / pin-project or tokio::pin!() instead of manual unsafe Pin projections.".to_string(),
                    });
                }
            }

            // =========================================================
            // 🌐 RÈGLES WEB3 & CRYPTOGRAPHIE (MULTI-LANGAGES: RUST / TS / JS / SOL)
            // =========================================================
            if (is_rust || is_js_ts || lower_path.ends_with(".sol")) && !is_test_file {
                // CVE-WEB3-REENTRANCY-UNCHECKED (CWE-841)
                if is_web3_mmo
                    && (trimmed.contains(".transfer(") || trimmed.contains("sendTransaction(") || trimmed.contains("send_transaction(") || (is_js_ts && (trimmed.contains("balance -=") || trimmed.contains("coins -="))))
                    && !content.contains("validate_")
                    && !content.contains("ReentrancyGuard")
                    && !content.contains("nonReentrant")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-REENTRANCY-UNCHECKED".to_string(),
                        cve_id: "CWE-841 / REENTRANCY".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Asset transfer or balance update executed without ownership validation or ReentrancyGuard".to_string(),
                        transmitted_data: Some("transfer_amount / asset_id".to_string()),
                        transmitted_type: Some("BigNumber / u64 (Unchecked Asset Balance Delta)".to_string()),
                        remediation: "Apply Checks-Effects-Interactions, use ReentrancyGuard, and validate signatures with validate_*().".to_string(),
                    });
                }

                // CVE-WEB3-SIG-MALLEABILITY (CWE-347 / Signature Malleability)
                let is_ecdsa_context = (trimmed.contains("ecrecover(")
                    && !trimmed.contains("fn ecrecover("))
                    || trimmed.contains("recoverAddress(")
                    || trimmed.contains("recover_address(")
                    || trimmed.contains("verifyMessage(")
                    || (trimmed.contains("verify_signature(")
                        && (content.contains("secp256k1")
                            || content.contains("ecrecover")
                            || content.contains("recover_address")
                            || content.contains("AuthVerifySignature")
                            || norm_path.contains("auth")));
                if is_ecdsa_context
                    && !content.contains("secp256k1n")
                    && !content.contains("0x7FFFFFFF")
                    && !content.contains("normalize_s")
                    && !content.contains("malleability_flag")
                    && !content.contains("s <=")
                    && !content.contains("s <")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-SIG-MALLEABILITY".to_string(),
                        cve_id: "CWE-347 / ECDSA-MALLEABILITY".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "ECDSA signature verified without enforcing low-S upper bound (s <= secp256k1n/2) allows malleability replay".to_string(),
                        transmitted_data: Some("signature_s_scalar".to_string()),
                        transmitted_type: Some("bytes32 / [u8; 32] (Malleable High-S Signature)".to_string()),
                        remediation: "Enforce low-S normalization (sig.normalize_s() or s <= 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D57617F1A13DDFF7FF15B81A839C635).".to_string(),
                    });
                }

                // CVE-WEB3-UNVALIDATED-CHAINID (CWE-294 / Cross-Chain Replay)
                if (trimmed.contains("verifyTypedData(") || trimmed.contains("hashTypedData(") || trimmed.contains("EIP712Domain") || trimmed.contains("AuthVerifySignature"))
                    && !content.contains("chainId")
                    && !content.contains("chain_id")
                    && !content.contains("BLOCK_CHAINID")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-UNVALIDATED-CHAINID".to_string(),
                        cve_id: "CWE-294 / CROSS-CHAIN-REPLAY".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Signature verification payload lacks chainId / verifyingContract domain binding (vulnerable to cross-chain replays)".to_string(),
                        transmitted_data: Some("eip712_domain".to_string()),
                        transmitted_type: Some("Struct (Missing ChainId in EIP-712 Domain)".to_string()),
                        remediation: "Add chain_id: u64 and verifying_contract to the signature domain separator and validate against active chain.".to_string(),
                    });
                }

                // CVE-WEB3-NONCE-RACE (CWE-362 / TOCTOU Nonce Race)
                if (lower_path.contains("auth") || lower_path.contains("session"))
                    && (trimmed.contains("consume_nonce") || trimmed.contains("invalidate_nonce") || trimmed.contains("del_nonce"))
                    && lines.iter().take(idx).any(|l| l.contains("session_token =") || l.contains("set_session(") || l.contains("create_session"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-NONCE-RACE".to_string(),
                        cve_id: "CWE-362 / TOCTOU-NONCE".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Nonce invalidated after session issuance creates race condition window (Time-Of-Check to Time-Of-Use)".to_string(),
                        transmitted_data: Some("auth_nonce".to_string()),
                        transmitted_type: Some("String / Nonce (Post-Action Invalidation)".to_string()),
                        remediation: "Atomically consume and delete the nonce (e.g. GETDEL in Redis or HashMap::remove) BEFORE creating user session.".to_string(),
                    });
                }

                // CVE-WEB3-AUTH-BYPASS-MOCK (CWE-287 / Cryptographic Signature Bypass or Zero-Address Return)
                let is_doc_catalog_line = trimmed.starts_with("attack:")
                    || trimmed.starts_with("root_cause:")
                    || trimmed.starts_with("description:")
                    || trimmed.starts_with("shield:")
                    || trimmed.starts_with("remediation:");
                if !is_doc_catalog_line
                    && (trimmed.contains("0xmock")
                        || trimmed.contains("mock_sig")
                        || trimmed.contains("bypass_sig")
                        || (trimmed.contains("ecrecover(")
                            && !trimmed.contains("fn ecrecover(")
                            && !content.contains("address(0)")
                            && !content.contains("0x0000000000000000000000000000000000000000")
                            && !content.contains("ZERO_ADDRESS")))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-AUTH-BYPASS-MOCK".to_string(),
                        cve_id: "CWE-287 / CRYPTO-SIGNATURE-BYPASS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Cryptographic signature verification bypass (mock signature prefix or ecrecover without zero-address 0x0 rejection) allows arbitrary wallet takeover".to_string(),
                        transmitted_data: Some("forged_or_mock_signature".to_string()),
                        transmitted_type: Some("HexSignature / Address (Unverified Signer Identity)".to_string()),
                        remediation: "Remove all mock signature prefixes, enforce real secp256k1 ecrecover/verifyMessage, and assert recovered_signer != 0x0000...0000.".to_string(),
                    });
                }

                // CVE-WEB3-ORACLE-SPOT-MANIPULATION (CWE-682 / Flash Loan AMM Spot Price Manipulation)
                if (trimmed.contains("getReserves()") || trimmed.contains(".getReserves()") || trimmed.contains("slot0()") || (trimmed.contains("balanceOf(") && trimmed.contains("pair")))
                    && !content.contains("consult(")
                    && !content.contains("AggregatorV3Interface")
                    && !content.contains("latestRoundData")
                    && !content.contains("TWAP")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-ORACLE-SPOT-MANIPULATION".to_string(),
                        cve_id: "CWE-682 / FLASH-LOAN-ORACLE-MANIPULATION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Reading spot prices directly from AMM reserves (getReserves/slot0) without TWAP or Chainlink feed: vulnerable to single-block Flash Loan price manipulation".to_string(),
                        transmitted_data: Some("amm_spot_reserves".to_string()),
                        transmitted_type: Some("uint112 / uint160 (Unbuffered Spot Price)".to_string()),
                        remediation: "Use decentralized price oracles (Chainlink latestRoundData with heartbeat check) or multi-block Uniswap v3 TWAP instead of instantaneous spot reserves.".to_string(),
                    });
                }

                // CVE-WEB3-TX-ORIGIN-PHISHING (CWE-302 / Authorization Bypass via tx.origin)
                if trimmed.contains("tx.origin ==") || trimmed.contains("tx.origin !=") || trimmed.contains("== tx.origin") || trimmed.contains("!= tx.origin") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-TX-ORIGIN-PHISHING".to_string(),
                        cve_id: "CWE-302 / TX-ORIGIN-AUTHORIZATION-BYPASS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Authentication or authorization check relies on tx.origin instead of msg.sender: vulnerable to phishing smart contract proxy attacks".to_string(),
                        transmitted_data: Some("tx_origin_address".to_string()),
                        transmitted_type: Some("address (Vulnerable Origin Invariant)".to_string()),
                        remediation: "Replace tx.origin with msg.sender or OpenZeppelin's _msgSender() to prevent intermediate contract authorization hijacking.".to_string(),
                    });
                }

                // CVE-WEB3-UNCHECKED-TRANSFER-RETURN (CWE-252 / Silent Transfer Failure for Non-Standard ERC20 like USDT)
                if (trimmed.contains(".transfer(") || trimmed.contains(".transferFrom("))
                    && !trimmed.contains("function transfer(")
                    && !trimmed.contains("function transferFrom(")
                    && !trimmed.contains("function transfer_")
                    && !trimmed.contains("super.transfer(")
                    && !trimmed.contains("super.transferFrom(")
                    && !trimmed.ends_with(';')
                    && !content.contains("SafeERC20")
                    && !content.contains("safeTransfer")
                    && !content.contains("safeTransferFrom")
                    && !trimmed.contains("require(")
                    && !trimmed.contains("bool success")
                    && (lower_path.contains("contract") || lower_path.contains("vault") || lower_path.contains("token") || lower_path.ends_with(".sol"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-UNCHECKED-TRANSFER-RETURN".to_string(),
                        cve_id: "CWE-252 / SILENT-TRANSFER-FAILURE-USDT".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Raw ERC20 transfer() or transferFrom() called without SafeERC20: non-standard tokens (USDT, BNB) revert or fail silently, allowing phantom deposits".to_string(),
                        transmitted_data: Some("erc20_transfer_call".to_string()),
                        transmitted_type: Some("bool (Unchecked Return Value)".to_string()),
                        remediation: "Use OpenZeppelin's SafeERC20: token.safeTransfer(to, amount) and token.safeTransferFrom(from, to, amount).".to_string(),
                    });
                }

                // CVE-WEB3-PERMIT-FRONT-RUNNING (CWE-362 / ERC-2612 Mempool Signature Front-Running)
                if trimmed.contains(".permit(")
                    && (content.contains(".transferFrom(") || content.contains("deposit("))
                    && !content.contains("try ")
                    && !content.contains("catch")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-PERMIT-FRONT-RUNNING".to_string(),
                        cve_id: "CWE-362 / MEMPOOL-SIGNATURE-GRIEFING".to_string(),
                        severity: "HIGH".to_string(),
                        message: "ERC-2612 permit() call executed without try/catch: mempool bots can front-run the signature, causing the user transaction to revert permanently (DoS Griefing)".to_string(),
                        transmitted_data: Some("permit_signature_v_r_s".to_string()),
                        transmitted_type: Some("bytes (Mempool Sniffable Signature)".to_string()),
                        remediation: "Wrap token.permit() in a try/catch block so execution proceeds normally if allowance is already satisfied by a front-running transaction.".to_string(),
                    });
                }

                // CVE-WEB3-HOOK-REENTRANCY (CWE-841 / Reentrancy via ERC-777 / ERC-1363 Callbacks)
                if (trimmed.contains("tokensReceived") || trimmed.contains("IERC777") || trimmed.contains("IERC1363") || trimmed.contains("transferAndCall"))
                    && !content.contains("nonReentrant")
                    && !content.contains("ReentrancyGuard")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-HOOK-REENTRANCY".to_string(),
                        cve_id: "CWE-841 / ERC777-ERC1363-HOOK-REENTRANCY".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "ERC-777/ERC-1363 token operations trigger automatic recipient hooks (tokensReceived) without ReentrancyGuard: vulnerable to invisible token reentrancy".to_string(),
                        transmitted_data: Some("erc777_hook_callback".to_string()),
                        transmitted_type: Some("Callback (Untrusted Hook Reentrancy)".to_string()),
                        remediation: "Apply OpenZeppelin's ReentrancyGuard nonReentrant modifier to all token deposit and withdrawal handlers.".to_string(),
                    });
                }

                // CVE-VM-ACCOUNT-MISSING-OWNER (CWE-284 / Missing Account Owner Check)
                if (trimmed.contains("AccountInfo") || trimmed.contains("ctx.accounts."))
                    && (trimmed.contains("data.borrow()") || trimmed.contains("try_borrow_data"))
                    && !content.contains("owner == program_id")
                    && !content.contains("owner =")
                    && !content.contains("has_one =")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-VM-ACCOUNT-MISSING-OWNER".to_string(),
                        cve_id: "CWE-284 / VM-ACCOUNT-OWNER-CONFUSION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Account data deserialized without verifying account.owner == program_id: attacker can forge fake accounts from arbitrary programs".to_string(),
                        transmitted_data: Some("vm_account_info".to_string()),
                        transmitted_type: Some("AccountInfo (Unchecked Program Owner)".to_string()),
                        remediation: "Verify account.owner == expected_program_id before deserializing or use a framework-level owner constraint.".to_string(),
                    });
                }

                // CVE-WEB3-READONLY-REENTRANCY (CWE-841 / Read-Only Reentrancy on Curve/Balancer LP Virtual Price)
                if (trimmed.contains("get_virtual_price()") || trimmed.contains(".get_virtual_price()") || trimmed.contains("calc_token_amount("))
                    && !content.contains("reentrancy")
                    && !content.contains("is_locked")
                    && !content.contains("withdraw_admin_fees")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-READONLY-REENTRANCY".to_string(),
                        cve_id: "CWE-841 / READONLY-REENTRANCY-LP-ORACLE".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Reading LP token virtual price in view function during unfinished pool withdrawals yields corrupted pricing".to_string(),
                        transmitted_data: Some("curve_virtual_price".to_string()),
                        transmitted_type: Some("uint256 (Deflated LP Virtual Price)".to_string()),
                        remediation: "Verify pool reentrancy lock status (e.g. check Curve pool lock or call non-reentrant state-touching check) before consuming LP virtual prices.".to_string(),
                    });
                }

                // CVE-WEB3-DIRTY-MEMORY-ASSEMBLY (CWE-682 / Solidity Dirty High-Order Bits in Inline Assembly)
                if (trimmed.starts_with("assembly") || trimmed.contains("assembly {") || trimmed.contains("assembly{"))
                    && (content.contains("bool") || content.contains("uint8"))
                    && !content.contains("0xff")
                    && !content.contains("0xFF")
                    && !content.contains("delegatecall")
                    && !content.contains("memory-safe")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-DIRTY-MEMORY-ASSEMBLY".to_string(),
                        cve_id: "CWE-682 / CWE-119-DIRTY-BITS-ASSEMBLY".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Inline assembly memory access on sub-32-byte types (bool, uint8, address) without bitwise masking leaves dirty higher-order bits, corrupting conditional branches".to_string(),
                        transmitted_data: Some("assembly_memory_slot".to_string()),
                        transmitted_type: Some("bytes32 (Uncleaned High-Order Bits)".to_string()),
                        remediation: "Clean higher-order bits explicitly using and(val, 0xff) for bool/uint8 or and(val, 0xffffffffffffffffffffffffffffffffffffffff) for addresses.".to_string(),
                    });
                }

                // CVE-WEB3-ERC4626-INFLATION-ATTACK (CWE-682 / ERC-4626 Vault First-Depositor Donation Inflation)
                if (trimmed.contains("ERC4626") || trimmed.contains("convertToShares(") || trimmed.contains("_convertToShares("))
                    && (trimmed.contains("totalSupply") || trimmed.contains("totalAssets"))
                    && !content.contains("_decimalsOffset")
                    && !content.contains("decimalsOffset")
                    && !content.contains("10 **")
                    && !content.contains("10**")
                    && !content.contains("dead_shares")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-ERC4626-INFLATION-ATTACK".to_string(),
                        cve_id: "CWE-682 / ERC4626-FIRST-DEPOSITOR-DONATION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "ERC-4626 share conversion calculated without virtual shares/assets offset: first depositor can front-run donations to truncate subsequent user deposits to 0 shares".to_string(),
                        transmitted_data: Some("vault_share_ratio".to_string()),
                        transmitted_type: Some("uint256 (Zero-Offset Inflation Vulnerability)".to_string()),
                        remediation: "Implement OpenZeppelin v4.9+ virtual shares offset via _decimalsOffset() or mint minimum dead shares (1000 wei) to address(0xdead) on initialization.".to_string(),
                    });
                }

                // CVE-WEB3-VAULT-ROUNDING-DIRECTION (CWE-682 / ERC-4626 Reverse Conversion Rounding)
                if (trimmed.contains("convertToShares") || trimmed.contains("convertToAssets") || trimmed.contains("previewDeposit") || trimmed.contains("previewMint") || trimmed.contains("previewWithdraw") || trimmed.contains("previewRedeem"))
                    && (trimmed.contains(" / ") || trimmed.contains("div(") || trimmed.contains("Rounding.Up") || trimmed.contains("Rounding.Ceil"))
                    && !content.contains("Math.Rounding.Floor")
                    && !content.contains("Rounding.Floor")
                    && !content.contains("Rounding.Down")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-VAULT-ROUNDING-DIRECTION".to_string(),
                        cve_id: "CWE-682 / INCORRECT-CONVERSION-ROUNDING".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "ERC-4626 conversion calculates division without explicit floor/down rounding or rounds up in user deposit: allows micro-minting inflation exploitation".to_string(),
                        transmitted_data: Some("conversion_rounding_direction".to_string()),
                        transmitted_type: Some("uint256 (Vulnerable Integer Division Rounding)".to_string()),
                        remediation: "Use OpenZeppelin Math.mulDiv(..., Math.Rounding.Floor) for deposit/convertToShares and Math.Rounding.Ceil for withdraw/previewWithdraw.".to_string(),
                    });
                }

                // CVE-WEB3-STALE-ORACLE-HEARTBEAT (CWE-682 / Stale Oracle Round Data)
                if trimmed.contains("latestRoundData(")
                    && !trimmed.contains("function latestRoundData")
                    && !is_interface_or_types
                    && !content.contains("updatedAt != 0")
                    && !content.contains("updatedAt > 0")
                    && !content.contains("block.timestamp -")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-STALE-ORACLE-HEARTBEAT".to_string(),
                        cve_id: "CWE-682 / STALE-ORACLE-ROUND-DATA".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Chainlink latestRoundData() queried without freshness check: missing validation on updatedAt > 0 and heartbeat time window (block.timestamp - updatedAt <= MAX_STALENESS)".to_string(),
                        transmitted_data: Some("oracle_round_answer".to_string()),
                        transmitted_type: Some("int256 (Stale Unchecked Price Feed)".to_string()),
                        remediation: "Validate round freshness: require(updatedAt > 0 && block.timestamp - updatedAt <= HEARTBEAT && answeredInRound >= roundId, 'Stale price feed').".to_string(),
                    });
                }

                // CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR (CWE-284 / Unprotected Flash Loan Callback)
                if (trimmed.contains("function onFlashLoan(") || trimmed.contains("function uniswapV3SwapCallback(") || trimmed.contains("function uniswapV3FlashCallback("))
                    && !trimmed.ends_with(';')
                    && !lower_path.contains("interface")
                    && !content.contains("interface ")
                    && !content.contains("msg.sender ==")
                    && !content.contains("== msg.sender")
                    && !content.contains("onlyPool")
                    && !content.contains("onlyOwner")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR".to_string(),
                        cve_id: "CWE-284 / UNPROTECTED-FLASHLOAN-CALLBACK".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Flash loan callback is external/public without verifying msg.sender == authorizedPool: any caller can trigger unauthorized callback token transfers".to_string(),
                        transmitted_data: Some("flashloan_caller_address".to_string()),
                        transmitted_type: Some("address (Unverified Callback Caller)".to_string()),
                        remediation: "Add an access control invariant: require(msg.sender == authorizedLenderPool, 'Unauthorized flash loan callback caller').".to_string(),
                    });
                }

                // CVE-WEB3-SIGNATURE-DEADLINE-MISSING (CWE-613 / Missing Signature Deadline)
                if (trimmed.contains("ecrecover(") || trimmed.contains("ECDSA.recover(") || trimmed.contains(".recover("))
                    && !trimmed.contains("//")
                    && (content.contains("function permit") || content.contains("bytes memory signature") || content.contains("bytes calldata signature") || content.contains("bytes32 r"))
                    && !content.contains("deadline")
                    && !content.contains("validUntil")
                    && !content.contains("block.timestamp")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-SIGNATURE-DEADLINE-MISSING".to_string(),
                        cve_id: "CWE-613 / MISSING-SIGNATURE-DEADLINE".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Cryptographic signature accepted without timestamp deadline or expiration check: vulnerable to delayed execution holding attacks by relayers/miners".to_string(),
                        transmitted_data: Some("ecdsa_signature_packet".to_string()),
                        transmitted_type: Some("bytes (Undated Cryptographic Authorization)".to_string()),
                        remediation: "Include an explicit deadline parameter in the signature hash and verify: require(block.timestamp <= deadline, 'Signature expired').".to_string(),
                    });
                }

                // CVE-WEB3-HOOK-DEFERRED-CHECKS-DESYNC (CWE-841 / Transient Liquidity Invariant Violation)
                if (lower_path.contains("hook") || lower_path.contains("target") || content.contains("HookTarget") || content.contains("IHookTarget"))
                    && (trimmed.contains("function checkVaultStatus") || trimmed.contains("function preOperationHook") || trimmed.contains("function postOperationHook") || trimmed.contains("function checkOperation"))
                    && !trimmed.ends_with(';')
                    && !trimmed.contains("super.")
                    && !lower_path.contains("interface")
                    && !content.contains("interface ")
                    && (content.contains(".call(") || content.contains(".transfer(") || content.contains(".transferFrom(") || content.contains("balanceOf("))
                    && !content.contains("isChecksDeferred")
                    && !content.contains("areChecksInProgress")
                    && !content.contains("onlyEVC")
                    && !content.contains("onlyVault")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-HOOK-DEFERRED-CHECKS-DESYNC".to_string(),
                        cve_id: "CWE-841 / TRANSIENT-LIQUIDITY-INVARIANT-VIOLATION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "HookTarget executes external calls or queries vault balance without checking EVC deferred check status: allows reentrancy and state bypass during deferred liquidity checks".to_string(),
                        transmitted_data: Some("hook_transient_execution".to_string()),
                        transmitted_type: Some("ExternalCall / StateRead (Unchecked EVC Deferred State)".to_string()),
                        remediation: "Verify EVC state: require(!evc.isChecksDeferred(account) || evc.areChecksInProgress(), 'Checks deferred in progress').".to_string(),
                    });
                }

                // CVE-WEB3-DECIMAL-SCALING-TRUNCATION (CWE-682 / Oracle & Collateral Decimal Scaling Underflow)
                let has_div_before_mul = if let Some(div_pos) = trimmed.find(" / 10**").or_else(|| trimmed.find(" / 1e")) {
                    if let Some(mul_pos) = trimmed.rfind('*') {
                        mul_pos > div_pos
                    } else {
                        false
                    }
                } else {
                    false
                };
                let has_static_sub_18 = trimmed.contains("10 ** (18 -") || trimmed.contains("10**(18 -") || trimmed.contains("10 ** (18-") || trimmed.contains("10**(18-");

                if (lower_path.contains("oracle") || lower_path.contains("price") || lower_path.contains("collateral") || lower_path.contains("vault"))
                    && (has_static_sub_18 || has_div_before_mul)
                    && !content.contains("mulDiv")
                    && !content.contains("FixedPointMathLib")
                    && !content.contains("FullMath")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-DECIMAL-SCALING-TRUNCATION".to_string(),
                        cve_id: "CWE-682 / ORACLE-DECIMAL-SCALING-UNDERFLOW".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Static 18-decimal scaling assumption or division before multiplication in price normalization: vulnerable to arithmetic underflow (tokens > 18 decimals) or dust precision truncation (WBTC/USDC)".to_string(),
                        transmitted_data: Some("decimal_normalized_amount".to_string()),
                        transmitted_type: Some("uint256 (Truncated / Underflowed Scale Factor)".to_string()),
                        remediation: "Use PRBMath or OpenZeppelin Math.mulDiv(price, targetScale, baseScale, Rounding) and handle both decimals <= 18 and decimals > 18.".to_string(),
                    });
                }

                // CVE-WEB3-FORK-REWARD-STAKING-DILUTION (CWE-682 / CWE-362 Reward Harvest Before Update)
                if (trimmed.contains("rewardToken.transfer(") || trimmed.contains("rewardToken.safeTransfer(") || trimmed.contains("distributeReward(") || trimmed.contains("_mintReward("))
                    && (content.contains("function deposit") || content.contains("function withdraw") || content.contains("function stake"))
                    && !content.contains("updatePool")
                    && !content.contains("accRewardPerShare")
                    && !content.contains("lastRewardBlock")
                    && !content.contains("lastRewardTime")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-FORK-REWARD-STAKING-DILUTION".to_string(),
                        cve_id: "CWE-682 / REWARD-HARVEST-BEFORE-UPDATE".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Reward distribution in staking/vault fork executed without atomic reward accumulator checkpoint (updatePool / accRewardPerShare): flash loan deposit can instantly drain accumulated rewards".to_string(),
                        transmitted_data: Some("fork_reward_payout".to_string()),
                        transmitted_type: Some("uint256 (Flash-Loan Drainable Yield)".to_string()),
                        remediation: "Always checkpoint the reward accumulator (updatePool() updating accRewardPerShare and lastRewardBlock) BEFORE updating user balances or paying rewards.".to_string(),
                    });
                }

                // CVE-WEB3-EIP1153-TRANSIENT-COMPOSABILITY-LEAK (EVM-CANCUN-CRITICAL / Dirty TSTORE Composability Pollution)
                if (trimmed.contains("tstore(") || (trimmed.contains(" transient ") && trimmed.contains('=')))
                    && !trimmed.contains(", 0)")
                    && !content.contains(", 0)")
                    && !content.contains("= 0;")
                    && !content.contains("= address(0);")
                    && !content.contains("delete ")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-EIP1153-TRANSIENT-COMPOSABILITY-LEAK".to_string(),
                        cve_id: "EVM-CANCUN-CRITICAL / TSTORE-UNCLEARED-COMPOSABILITY-POLLUTION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "EIP-1153 transient storage written via tstore() or transient variable without explicit zero-reset before call exit: subsequent sub-calls in a multicall/batch/bundler transaction inherit dirty authenticated state or flash accounting deltas".to_string(),
                        transmitted_data: Some("eip1153_dirty_tstore_slot".to_string()),
                        transmitted_type: Some("bytes32 (Uncleared Transaction-Wide Transient Slot)".to_string()),
                        remediation: "Always clear EIP-1153 transient storage slots (tstore(slot, 0) or delete transientVar) at the end of the execution frame to preserve multi-call composability.".to_string(),
                    });
                }

                // CVE-WEB3-EIP7702-EOA-CODE-BYPASS-COLLISION (EVM-PECTRA-CRITICAL / EIP-7702 EOA Code Bypass & Slot Collision)
                if (trimmed.contains(".code.length == 0")
                    || trimmed.contains("extcodesize(")
                    || trimmed.contains("!isContract(msg.sender)")
                    || trimmed.contains("isContract(msg.sender) == false")
                    || trimmed.contains("0xef0100"))
                    && (trimmed.contains("require(")
                        || trimmed.contains("if (")
                        || trimmed.contains("if(")
                        || trimmed.contains("iszero(extcodesize")
                        || trimmed.contains("0xef0100"))
                    && !content.contains("erc7201")
                    && !content.contains("ERC7201")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-EIP7702-EOA-CODE-BYPASS-COLLISION".to_string(),
                        cve_id: "EVM-PECTRA-CRITICAL / EIP7702-EXTCODESIZE-BYPASS-AND-SLOT-COLLISION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Anti-bot/EOA check relying on extcodesize == 0 / msg.sender.code.length == 0 or EIP-7702 delegation without ERC-7201 namespaced storage: bypassed by contract constructors and EIP-7702 0xef0100 EOA code delegation, risking slot-0 storage collision".to_string(),
                        transmitted_data: Some("eoa_delegation_code_check".to_string()),
                        transmitted_type: Some("address / bytes (EIP-7702 EOA Delegation & Slot-0 Collision)".to_string()),
                        remediation: "Never rely on extcodesize == 0 or code.length == 0 for security invariants, and enforce ERC-7201 (@custom:storage-location erc7201:...) namespaced storage on all EIP-7702 delegatable account contracts.".to_string(),
                    });
                }

                // CVE-WEB3-ERC4337-PAYMASTER-GAS-SIPHON (AA-ERC4337-CRITICAL / Paymaster postOp Gas Griefing Drain)
                if (trimmed.contains("function validatePaymasterUserOp(")
                    || trimmed.contains("function _postOp(")
                    || trimmed.contains("function postOp("))
                    && !trimmed.ends_with(';')
                    && !is_interface_or_types
                    && (((trimmed.contains("_postOp(") || trimmed.contains("postOp("))
                        && (content.contains(".transferFrom(") || content.contains(".safeTransferFrom("))
                        && !content.contains("preCharge")
                        && !content.contains("prefundToken"))
                        || (trimmed.contains("validatePaymasterUserOp(")
                            && !content.contains("maxCost <=")
                            && !content.contains("maxCost <")
                            && !content.contains("MAX_COST")
                            && !content.contains("postOpGasLimit")))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-ERC4337-PAYMASTER-GAS-SIPHON".to_string(),
                        cve_id: "AA-ERC4337-CRITICAL / PAYMASTER-POSTOP-UNBOUNDED-GAS-GRIEFING".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "ERC-4337 Paymaster validates UserOp without bounding maxCost/postOpGasLimit or defers ERC-20 transferFrom to postOp without pre-escrow in validatePaymasterUserOp: attacker can revoke allowance during inner call and drain the Paymaster's EntryPoint ETH deposit".to_string(),
                        transmitted_data: Some("userop_paymaster_gas_budget".to_string()),
                        transmitted_type: Some("UserOperation (Unescrowed postOp Gas Liability)".to_string()),
                        remediation: "Pre-escrow the maximum ERC-20 token cost inside validatePaymasterUserOp() (refunding excess in _postOp) and strictly bound maxCost and verification/postOp gas limits.".to_string(),
                    });
                }

                // CVE-WEB3-OMNICHAIN-LZ-CHANNEL-BRICKING (OMNICHAIN-CRITICAL / Cross-Chain Uncaught Revert Fund Burn)
                if (trimmed.contains("function _lzReceive(")
                    || trimmed.contains("function lzReceive(")
                    || trimmed.contains("function _ccipReceive(")
                    || trimmed.contains("function receiveWormholeMessages("))
                    && !trimmed.ends_with(';')
                    && !is_interface_or_types
                    && (content.contains(".transfer(")
                        || content.contains(".safeTransfer(")
                        || content.contains("_mint(")
                        || content.contains(".call")
                        || content.contains("abi.decode("))
                    && !content.contains("try ")
                    && !content.contains("failedMessages")
                    && !content.contains("storedPayload")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-OMNICHAIN-LZ-CHANNEL-BRICKING".to_string(),
                        cve_id: "OMNICHAIN-CRITICAL / CROSS-CHAIN-UNCAUGHT-REVERT-FUND-BURN".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Cross-chain receiver (_lzReceive / _ccipReceive) executes token transfer, mint, or external hook without try/catch quarantine (failedMessages / storedPayload): any revert on destination chain permanently burns source-chain assets or bricks the ordered messaging channel".to_string(),
                        transmitted_data: Some("crosschain_inbound_payload".to_string()),
                        transmitted_type: Some("bytes (Unquarantined Omnichain Message Payload)".to_string()),
                        remediation: "Isolate payload execution in an external self-call wrapped in try/catch and store failed payloads in failedMessages[guid] = keccak256(payload) for manual retry.".to_string(),
                    });
                }

                // CVE-WEB3-EIP150-63-64-GAS-STIPEND-GRIEFING (EVM-INTERNALS-CRITICAL / EIP-150 63/64th Gas Stipend Subcall OOG Griefing)
                if (trimmed.contains(".call(") || trimmed.contains(".call{") || (trimmed.contains("try ") && trimmed.contains("catch")))
                    && (content.contains("executed[") || content.contains("processed[") || content.contains("nonces[") || content.contains("relayed["))
                    && !content.contains("gasleft()")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-EIP150-63-64-GAS-STIPEND-GRIEFING".to_string(),
                        cve_id: "EVM-INTERNALS-CRITICAL / EIP150-63-64TH-SUBCALL-OOG-SILENT-BYPASS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Sub-call (.call / try-catch) in relayer/executor consumes nonce or marks message executed without checking gasleft(): under EIP-150 (63/64 rule), a relayer can starve the sub-call to fail Out-Of-Gas while the parent frame survives on the remaining 1/64th gas and steals the execution reward".to_string(),
                        transmitted_data: Some("eip150_subcall_gas_stipend".to_string()),
                        transmitted_type: Some("uint256 (Unchecked 63/64th Sub-Call Gas Margin)".to_string()),
                        remediation: "Assert sufficient gas before/after the sub-call: require(gasleft() > (targetGas * 64) / 63 + GAS_BUFFER, 'EIP-150 insufficient gas for subcall').".to_string(),
                    });
                }

                // CVE-WEB3-ZK-SNARK-FIELD-OVERFLOW-ALIASING (ZK-CRYPTO-CRITICAL / BN254 Scalar Field Modulo Aliasing 5x Spend)
                if (trimmed.contains("verifyProof(") || trimmed.contains("function verify(") || (trimmed.contains("nullifier") && trimmed.contains("uint256")))
                    && (content.contains("nullifier") || content.contains("publicInputs") || content.contains("input["))
                    && !trimmed.ends_with(';')
                    && !is_interface_or_types
                    && !content.contains("SNARK_SCALAR_FIELD")
                    && !content.contains("SCALAR_FIELD")
                    && !content.contains("21888242871839275222246405745257275088548364400416034343698204186575808495617")
                    && !content.contains("0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-ZK-SNARK-FIELD-OVERFLOW-ALIASING".to_string(),
                        cve_id: "ZK-CRYPTO-CRITICAL / BN254-SCALAR-FIELD-MODULO-ALIASING".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "ZK-SNARK verifier accepts uint256 public inputs/nullifiers without enforcing < SNARK_SCALAR_FIELD (BN254 order r ≈ 2^254): attacker can add multiples of r (nullifier + k*r) to replay the same valid ZK proof up to 5 times".to_string(),
                        transmitted_data: Some("zk_snark_unreduced_public_input".to_string()),
                        transmitted_type: Some("uint256 (Unbounded Modulo-r Aliasing Scalar)".to_string()),
                        remediation: "Strictly enforce require(input[i] < SNARK_SCALAR_FIELD) (21888242871839275222246405745257275088548364400416034343698204186575808495617) on all public inputs and nullifiers.".to_string(),
                    });
                }

                // CVE-WEB3-MERKLE-SECOND-PREIMAGE-LEAF-NODE-COLLISION (CRYPTO-CRITICAL / 64-Byte Merkle Leaf Internal Node Collision)
                if (trimmed.contains("keccak256(abi.encode(") || trimmed.contains("keccak256(abi.encodePacked("))
                    && (content.contains("MerkleProof") || content.contains("merkleRoot") || lower_path.contains("merkle") || lower_path.contains("airdrop"))
                    && !trimmed.contains("bytes.concat(keccak256(")
                    && !content.contains("bytes.concat(keccak256(")
                    && !trimmed.contains("0x00")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-MERKLE-SECOND-PREIMAGE-LEAF-NODE-COLLISION".to_string(),
                        cve_id: "CRYPTO-CRITICAL / MERKLE-64BYTE-LEAF-INTERNAL-NODE-SPOOF".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Single-hashed 64-byte Merkle leaf (keccak256(abi.encode(...))) without double hashing or 0x00 domain prefix: attacker can pass two 32-byte internal Merkle tree nodes as a forged leaf (Second Preimage Attack)".to_string(),
                        transmitted_data: Some("merkle_64byte_single_hashed_leaf".to_string()),
                        transmitted_type: Some("bytes32 (Unprefixed 64-Byte Leaf Preimage)".to_string()),
                        remediation: "Double-hash all Merkle leaves: bytes32 leaf = keccak256(bytes.concat(keccak256(abi.encode(account, amount))));".to_string(),
                    });
                }

                // CVE-WEB3-CREATE2-METAMORPHIC-REINIT-RUG (EVM-BYTECODE-CRITICAL / CREATE2 Unsalted Sender Front-Run & Metamorphic Rug)
                if (trimmed.contains("create2(") || trimmed.contains("{salt:") || trimmed.contains("{ salt:"))
                    && !content.contains("keccak256(abi.encode(msg.sender")
                    && !content.contains("keccak256(abi.encodePacked(msg.sender")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-WEB3-CREATE2-METAMORPHIC-REINIT-RUG".to_string(),
                        cve_id: "EVM-BYTECODE-CRITICAL / CREATE2-UNSALTED-SENDER-FRONT-RUN-AND-METAMORPHIC".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "CREATE2 deterministic deployment uses raw user salt without binding msg.sender (keccak256(abi.encode(msg.sender, salt))): vulnerable to mempool/cross-chain CREATE2 address hijacking and metamorphic initCode substitution".to_string(),
                        transmitted_data: Some("create2_unbound_salt".to_string()),
                        transmitted_type: Some("bytes32 (Unbound User-Controlled CREATE2 Salt)".to_string()),
                        remediation: "Bind the caller's address into the CREATE2 salt: bytes32 safeSalt = keccak256(abi.encode(msg.sender, userSalt));".to_string(),
                    });
                }
            }

            // =========================================================
            // 🎮🗄️ RÈGLES MMO IDOR & REDIS / CACHE SAST (MULTI-LANGAGES)
            // =========================================================
            if is_rust || is_js_ts || is_py {
                // CVE-MMO-IDOR-TRUSTED-USERID (CWE-639 / Unauthenticated Client userId Spoofing)
                if (lower_path.contains("server") || lower_path.contains("api") || lower_path.contains("controller") || lower_path.contains("routes"))
                    && ((trimmed.contains("const { userId") || trimmed.contains("let { userId") || trimmed.contains("req.body.userId") || trimmed.contains("body.user_id") || trimmed.contains("payload.user_id"))
                        && (trimmed.contains("body") || trimmed.contains("req.") || trimmed.contains("payload")))
                    && !content.contains("authorization")
                    && !content.contains("verify_session")
                    && !content.contains("session_token")
                    && !content.contains("sessionsCache")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-MMO-IDOR-TRUSTED-USERID".to_string(),
                        cve_id: "CWE-639 / IDOR-CLIENT-USERID-SPOOF".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Direct trust of client-supplied userId/user_id in request body without server-side session token resolution (IDOR / Player Spoofing)".to_string(),
                        transmitted_data: Some("client_supplied_user_id".to_string()),
                        transmitted_type: Some("String / UUID (Unauthenticated Actor Identity)".to_string()),
                        remediation: "Never trust userId from request payloads; extract Bearer token from Authorization header and resolve canonical userId from server session store.".to_string(),
                    });
                }

                // CVE-DB-REDIS-UNAUTH-KEYS-DOS (CWE-306 / CWE-400 Redis O(N) KEYS Block & Unauthenticated RESP Socket)
                if ((trimmed.contains("cmd === 'KEYS'") || trimmed.contains("cmd == \"KEYS\"") || trimmed.contains(".keys(\"*\")") || trimmed.contains(".keys('*')") || trimmed.contains("redis.keys("))
                    && !content.contains("SCAN"))
                    || ((lower_path.contains("redis") || lower_path.contains("cache"))
                        && (trimmed.contains("net.createServer(") || trimmed.contains("TcpListener::bind("))
                        && !content.contains("AUTH")
                        && !content.contains("requirepass")
                        && !content.contains("verify_auth"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-DB-REDIS-UNAUTH-KEYS-DOS".to_string(),
                        cve_id: "CWE-306 / CWE-400-REDIS-KEYS-DOS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Unauthenticated Redis/Cache TCP socket or blocking O(N) 'KEYS *' command execution freezes single-threaded cache worker and leaks session keys".to_string(),
                        transmitted_data: Some("redis_resp_command_stream".to_string()),
                        transmitted_type: Some("RESP Command (Unauthenticated Socket / O(N) Keyspace Scan)".to_string()),
                        remediation: "Require AUTH token before processing RESP commands, bind to 127.0.0.1, and replace O(N) KEYS with cursor-based SCAN.".to_string(),
                    });
                }
            }

            // =========================================================
            // 🐍 RÈGLES SPÉCIFIQUES PYTHON
            // =========================================================
            if is_py && !is_test_file {
                // CVE-PY-DESERIALIZE-RCE (CWE-502 / Remote Code Execution)
                if trimmed.contains("pickle.loads(") || trimmed.contains("pickle.load(") || trimmed.contains("_pickle.loads(") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-DESERIALIZE-RCE".to_string(),
                        cve_id: "CWE-502 / CVE-PY-DESERIALIZE-RCE".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Unsafe pickle deserialization: untrusted byte stream leads to direct Remote Code Execution (RCE)".to_string(),
                        transmitted_data: Some("pickle_payload".to_string()),
                        transmitted_type: Some("bytes (Untrusted Serialized Pickle Stream)".to_string()),
                        remediation: "Use json.loads() or Pydantic v2 schemas; never deserialize untrusted pickle data.".to_string(),
                    });
                } else if (trimmed.contains("yaml.load(") && !trimmed.contains("SafeLoader") && !trimmed.contains("CSafeLoader")) || trimmed.contains("yaml.unsafe_load(") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-YAML-UNSAFE-LOAD".to_string(),
                        cve_id: "CWE-502 / YAML-UNSAFE-DESERIALIZATION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Unsafe YAML loading: arbitrary Python object instantiation and code execution via yaml.load without SafeLoader or yaml.unsafe_load".to_string(),
                        transmitted_data: Some("yaml_stream".to_string()),
                        transmitted_type: Some("str / bytes (Unsafe YAML Object Instantiation)".to_string()),
                        remediation: "Use yaml.safe_load(data) or yaml.load(data, Loader=yaml.SafeLoader).".to_string(),
                    });
                } else if trimmed.contains("eval(") || trimmed.contains("exec(") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-DESERIALIZE-RCE".to_string(),
                        cve_id: "CWE-94 / CVE-PY-CODE-INJ".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Dynamic code evaluation via eval() or exec(): allows arbitrary remote code execution".to_string(),
                        transmitted_data: Some("dynamic_code".to_string()),
                        transmitted_type: Some("str (Arbitrary Executable Code)".to_string()),
                        remediation: "Refactor to use AST-based expression evaluators or safe data structures.".to_string(),
                    });
                }

                // CVE-PY-SHELL-INJ (CWE-78 / Shell Injection)
                if (trimmed.contains("shell=True") && trimmed.contains("subprocess"))
                    || trimmed.contains("os.system(")
                    || trimmed.contains("os.popen(")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-SHELL-INJ".to_string(),
                        cve_id: "CWE-78 / CVE-PY-SHELL-INJ".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Subprocess executed with shell=True or os.system: untrusted input allows shell command injection".to_string(),
                        transmitted_data: Some("cmd_string".to_string()),
                        transmitted_type: Some("str (Dynamic Shell Concatenation)".to_string()),
                        remediation: "Set shell=False and pass command as a list [\"cmd\", \"arg1\"], avoiding shell interpreters.".to_string(),
                    });
                }

                // CVE-PY-SSRF-UNVALIDATED (CWE-918 / SSRF & Untyped Dict)
                if (trimmed.contains("requests.get(") || trimmed.contains("requests.post(") || trimmed.contains("httpx.get(") || trimmed.contains("httpx.post("))
                    && !trimmed.contains("\"http") && !trimmed.contains("'http")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-SSRF-UNVALIDATED".to_string(),
                        cve_id: "CWE-918 / CVE-PY-SSRF-UNVALIDATED".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Outbound HTTP request to dynamic URL: potential Server-Side Request Forgery (SSRF)".to_string(),
                        transmitted_data: Some("target_url".to_string()),
                        transmitted_type: Some("str (Unvalidated Remote Target URL)".to_string()),
                        remediation: "Validate requested URL against an explicit allowlist and block private IP ranges (127.0.0.1, 169.254.169.254).".to_string(),
                    });
                } else if (trimmed.contains(": dict") || trimmed.contains("payload: dict") || trimmed.contains("body: dict"))
                    && (lower_path.contains("api") || lower_path.contains("server") || lower_path.contains("router"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-UNTYPED-PAYLOAD".to_string(),
                        cve_id: "CWE-20 / CVE-PY-UNTYPED-PAYLOAD".to_string(),
                        severity: "HIGH".to_string(),
                        message: "API route handler accepts raw untyped dict instead of a typed Pydantic contract from contracts/".to_string(),
                        transmitted_data: Some("raw_dict_payload".to_string()),
                        transmitted_type: Some("dict (Unvalidated & Untyped Input)".to_string()),
                        remediation: "Replace 'dict' with a strict Pydantic v2 model defined in contracts/schemas.py.".to_string(),
                    });
                }

                // CVE-PY-PATH-TRAVERSAL-ARCHIVE (CWE-22 / Path Traversal & ZipSlip Extraction)
                if (trimmed.contains(".extractall(")
                    || trimmed.contains("shutil.unpack_archive(")
                    || ((trimmed.contains("open(") || trimmed.contains("send_file(") || trimmed.contains("os.path.join("))
                        && (trimmed.contains("filename") || trimmed.contains("user_path") || trimmed.contains("file_path") || trimmed.contains("f\""))))
                    && !content.contains("is_relative_to")
                    && !content.contains(".resolve()")
                    && !content.contains("os.path.basename")
                    && !content.contains("secure_filename")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-PATH-TRAVERSAL-ARCHIVE".to_string(),
                        cve_id: "CWE-22 / PATH-TRAVERSAL-ZIPSLIP".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Unvalidated file path access or archive extraction (.extractall / open) allows arbitrary file read/overwrite via '../' Path Traversal or ZipSlip".to_string(),
                        transmitted_data: Some("untrusted_file_or_archive_path".to_string()),
                        transmitted_type: Some("str / Path (Unsanitized Relative Path)".to_string()),
                        remediation: "Canonicalize path via target = (BASE_DIR / user_path).resolve() and assert target.is_relative_to(BASE_DIR.resolve()) before opening or extracting.".to_string(),
                    });
                }

                // CVE-PY-JINJA2-SSTI-INJECTION (CWE-1336 / Server-Side Template Injection)
                if (trimmed.contains("render_template_string(") || trimmed.contains("jinja2.Template(") || (trimmed.contains("Template(") && content.contains("jinja2")))
                    && (trimmed.contains("f\"") || trimmed.contains("f'") || trimmed.contains(" + ") || trimmed.contains("%") || trimmed.contains(".format("))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-JINJA2-SSTI-INJECTION".to_string(),
                        cve_id: "CWE-1336 / JINJA2-SSTI-RCE".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Jinja2 Server-Side Template Injection (SSTI): dynamic string formatting in template compilation allows arbitrary Python code execution".to_string(),
                        transmitted_data: Some("template_string_expression".to_string()),
                        transmitted_type: Some("str (Dynamic Template Concatenation / SSTI)".to_string()),
                        remediation: "Pass dynamic values via template context kwargs render_template_string(TEMPLATE, user_val=val) instead of formatting the template string.".to_string(),
                    });
                }

                // CVE-PY-SQL-ALCHEMY-TEXT-INJECTION (CWE-89 / SQL Alchemy Raw Text Injection)
                if (trimmed.contains("text(f\"") || trimmed.contains("text(f'")
                    || (trimmed.contains("text(") && (trimmed.contains(" + ") || trimmed.contains(".format(") || trimmed.contains(" % "))))
                    && (content.contains("sqlalchemy") || content.contains("db.session") || content.contains("session.execute") || lower_path.contains("db") || lower_path.contains("sql") || lower_path.contains("model"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-SQL-ALCHEMY-TEXT-INJECTION".to_string(),
                        cve_id: "CWE-89 / SQLALCHEMY-RAW-TEXT-INJECTION".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Raw SQL clause in sqlalchemy.text() concatenated with dynamic variables: bypasses ORM escaping and allows direct SQL injection".to_string(),
                        transmitted_data: Some("raw_sql_text_expression".to_string()),
                        transmitted_type: Some("str (Unparameterized Raw SQL Clause)".to_string()),
                        remediation: "Use bound parameters: text('SELECT * FROM users WHERE id = :id').bindparams(id=val).".to_string(),
                    });
                }

                // CVE-PY-JWT-NONE-ALGORITHM (CWE-327 / JWT Authentication Bypass with alg:none)
                if trimmed.contains("jwt.decode(")
                    && (trimmed.contains("verify=False")
                        || trimmed.contains("\"verify_signature\": False")
                        || trimmed.contains("'verify_signature': False")
                        || trimmed.contains("\"none\"")
                        || trimmed.contains("'none'"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-JWT-NONE-ALGORITHM".to_string(),
                        cve_id: "CWE-327 / JWT-UNVERIFIED-NONE-ALGORITHM".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "JWT decoded with signature verification disabled or algorithm 'none' allowed: allows complete authentication bypass".to_string(),
                        transmitted_data: Some("jwt_token_payload".to_string()),
                        transmitted_type: Some("str (Unverified / Alg:None JWT Signature)".to_string()),
                        remediation: "Require secret and specify algorithms explicitly: jwt.decode(token, SECRET, algorithms=['HS256']).".to_string(),
                    });
                }

                // CVE-PY-PARAMIKO-MISSING-HOST-KEY (CWE-295 / AutoAddPolicy MITM)
                if (trimmed.contains("set_missing_host_key_policy(") || trimmed.contains("AutoAddPolicy("))
                    && (trimmed.contains("AutoAddPolicy") || content.contains("paramiko"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-PARAMIKO-MISSING-HOST-KEY".to_string(),
                        cve_id: "CWE-295 / PARAMIKO-AUTOADD-HOSTKEY-MITM".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Paramiko SSH client configured with AutoAddPolicy: silently trusts unknown host keys, enabling Man-In-The-Middle attacks".to_string(),
                        transmitted_data: Some("paramiko_ssh_policy".to_string()),
                        transmitted_type: Some("paramiko.AutoAddPolicy (Untrusted Host Key Policy)".to_string()),
                        remediation: "Use paramiko.RejectPolicy() and preload trusted host keys via client.load_system_host_keys().".to_string(),
                    });
                }

                // CVE-PY-XML-XXE-ENTITY-EXPANSION (CWE-611 / XML External Entity)
                if (trimmed.contains("xml.etree.ElementTree.fromstring(")
                    || trimmed.contains("ET.fromstring(")
                    || trimmed.contains("minidom.parse(")
                    || trimmed.contains("minidom.parseString(")
                    || (trimmed.contains("XMLParser(") && trimmed.contains("resolve_entities=True")))
                    && !content.contains("defusedxml")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-PY-XML-XXE-ENTITY-EXPANSION".to_string(),
                        cve_id: "CWE-611 / XML-EXTERNAL-ENTITY-XXE".to_string(),
                        severity: "HIGH".to_string(),
                        message: "XML parsed using standard library without defusedxml or entity disabling: vulnerable to XML External Entity (XXE) data exfiltration and Billion Laughs DoS".to_string(),
                        transmitted_data: Some("untrusted_xml_stream".to_string()),
                        transmitted_type: Some("str / bytes (Unsafe XML Document)".to_string()),
                        remediation: "Use defusedxml.minidom or defusedxml.ElementTree to parse untrusted XML safely.".to_string(),
                    });
                }
            }

            // =========================================================
            // 🟢 RÈGLES SPÉCIFIQUES NODE.JS / TYPESCRIPT
            // =========================================================
            if is_js_ts {
                // CVE-JS-EVENTLOOP-REDOS-JSON (CWE-1333 / CWE-770 Unbounded Stream & Event-Loop DoS)
                if ((trimmed.contains("req.on('data'") || trimmed.contains("req.on(\"data\"") || trimmed.contains("body += chunk"))
                    && !content.contains("MAX_BODY")
                    && !content.contains("maxBody")
                    && !content.contains("body.length >")
                    && !content.contains("req.destroy("))
                    || (trimmed.contains("new RegExp(")
                        && !trimmed.contains("new RegExp(\"^")
                        && !trimmed.contains("new RegExp('^")
                        && !content.contains("escapeRegExp"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-EVENTLOOP-REDOS-JSON".to_string(),
                        cve_id: "CWE-1333 / CWE-770-EVENTLOOP-DOS".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Unbounded HTTP stream accumulation (req.on('data') without byte cap) or dynamic RegExp blocks the single-threaded Node.js Event Loop / V8 heap".to_string(),
                        transmitted_data: Some("unbounded_http_chunk_or_regex".to_string()),
                        transmitted_type: Some("Buffer / RegExp (Uncapped Stream or Catastrophic Backtracking)".to_string()),
                        remediation: "Enforce a strict byte limit on incoming chunks (if (body.length > MAX_BODY_BYTES) return req.destroy()) and avoid dynamic RegExp on user input.".to_string(),
                    });
                }

                // CVE-JS-XSS-DOM (CWE-79 / Cross-Site Scripting)
                if trimmed.contains("dangerouslySetInnerHTML") || trimmed.contains(".innerHTML =") || trimmed.contains("document.write(") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-XSS-DOM".to_string(),
                        cve_id: "CWE-79 / CVE-JS-XSS-DOM".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Direct DOM injection via innerHTML / dangerouslySetInnerHTML allows stored or reflected XSS".to_string(),
                        transmitted_data: Some("html_payload".to_string()),
                        transmitted_type: Some("string (Unsanitized HTML Markup)".to_string()),
                        remediation: "Sanitize content with DOMPurify.sanitize() or use textContent and safe React JSX tags.".to_string(),
                    });
                }

                // CVE-JS-PROTO-POLLUTION & RCE (CWE-1321 / CWE-94)
                if trimmed.contains("eval(") || trimmed.contains("new Function(") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-CODE-INJ".to_string(),
                        cve_id: "CWE-94 / CVE-JS-CODE-INJ".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "eval() or new Function() with dynamic input leads to Remote Code Execution".to_string(),
                        transmitted_data: Some("eval_code".to_string()),
                        transmitted_type: Some("string (Arbitrary JavaScript Code)".to_string()),
                        remediation: "Remove eval() and use JSON.parse() or structured logic.".to_string(),
                    });
                } else if trimmed.contains("child_process.exec(") || trimmed.contains("execSync(") {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-CMD-INJ".to_string(),
                        cve_id: "CWE-78 / CVE-JS-CMD-INJ".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "child_process.exec spawns a shell and is vulnerable to command injection".to_string(),
                        transmitted_data: Some("shell_cmd".to_string()),
                        transmitted_type: Some("string (Shell Command String)".to_string()),
                        remediation: "Use child_process.execFile() or spawn() without shell: true.".to_string(),
                    });
                } else if (trimmed.contains(": any") || (trimmed.contains("req.body") && !trimmed.contains("parse") && !trimmed.contains("validate")))
                    && (lower_path.contains("api") || lower_path.contains("server") || lower_path.contains("routes"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-PROTO-POLLUTION".to_string(),
                        cve_id: "CWE-1321 / CVE-JS-PROTO-POLLUTION".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Untyped payload (any / req.body) processed without Zod schema: risk of prototype pollution".to_string(),
                        transmitted_data: Some("req.body / untyped_payload".to_string()),
                        transmitted_type: Some("any (Untyped Object / Prototype Exposure)".to_string()),
                        remediation: "Validate input using a Zod schema defined in contracts/ (e.g. BookingSchema.parse(req.body)).".to_string(),
                    });
                }

                // CVE-CLIENT-UNSAFE-ASSETS (CWE-829 / Unsafe Asset Loading)
                if (trimmed.contains("Assets.load(") || trimmed.contains("PIXI.Assets.load(") || trimmed.contains("PIXI.Texture.from("))
                    && (trimmed.contains("url") || trimmed.contains("params") || trimmed.contains("avatar") || trimmed.contains("mobi"))
                    && !content.contains("ALLOWLIST")
                    && !content.contains("trusted_domain")
                    && !content.contains("validate_url")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-CLIENT-UNSAFE-ASSETS".to_string(),
                        cve_id: "CWE-829 / UNSAFE-ASSET-LOADING".to_string(),
                        severity: "MEDIUM".to_string(),
                        message: "Dynamic WebGL/PixiJS texture loading without domain allowlist validation allows client SSRF or asset poisoning".to_string(),
                        transmitted_data: Some("texture_url".to_string()),
                        transmitted_type: Some("string (Unvalidated Image Asset URL)".to_string()),
                        remediation: "Assert asset URL starts with authorized CDN prefix or IPFS gateway origin before calling Assets.load().".to_string(),
                    });
                }

                // CVE-JS-VM-SANDBOX-ESCAPE (CWE-94 / Node.js vm Sandbox Escape RCE)
                if trimmed.contains("vm.runInContext(")
                    || trimmed.contains("vm.runInNewContext(")
                    || trimmed.contains("vm.runInThisContext(")
                    || (trimmed.contains("new vm.Script(") && !trimmed.contains("isolated-vm"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-VM-SANDBOX-ESCAPE".to_string(),
                        cve_id: "CWE-94 / VM-SANDBOX-ESCAPE-RCE".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Node.js vm module used for untrusted code: vm context is easily escapable via Function constructor, allowing RCE".to_string(),
                        transmitted_data: Some("untrusted_vm_script".to_string()),
                        transmitted_type: Some("string (Node.js vm Escapable Code)".to_string()),
                        remediation: "Do not use the Node.js vm module for untrusted execution; use isolated-vm or WebAssembly.".to_string(),
                    });
                }

                // CVE-JS-SSRF-DYNAMIC-AXIOS-FETCH (CWE-918 / Outbound SSRF via Axios / Fetch)
                if (trimmed.contains("axios.get(") || trimmed.contains("axios.post(") || trimmed.contains("fetch(") || trimmed.contains("got("))
                    && !trimmed.contains("\"http")
                    && !trimmed.contains("'http")
                    && !trimmed.contains("`http")
                    && (trimmed.contains("req.query") || trimmed.contains("req.body") || trimmed.contains("url") || trimmed.contains("targetUrl") || trimmed.contains("webhook"))
                    && !content.contains("isValidUrl")
                    && !content.contains("allowlist")
                    && !content.contains("whitelist")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-SSRF-DYNAMIC-AXIOS-FETCH".to_string(),
                        cve_id: "CWE-918 / AXIOS-FETCH-OUTBOUND-SSRF".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Outbound HTTP request (axios/fetch/got) to dynamic target URL without allowlist: enables Server-Side Request Forgery (SSRF)".to_string(),
                        transmitted_data: Some("dynamic_target_url".to_string()),
                        transmitted_type: Some("string (Unvalidated Target URL)".to_string()),
                        remediation: "Validate requested URL against an explicit allowlist and block private IP ranges (127.0.0.1, 169.254.169.254).".to_string(),
                    });
                }

                // CVE-JS-PATH-TRAVERSAL-SENDFILE (CWE-22 / Arbitrary File Read via res.sendFile)
                if (trimmed.contains("res.sendFile(") || trimmed.contains("fs.readFile(") || trimmed.contains("fs.createReadStream("))
                    && (trimmed.contains("req.params") || trimmed.contains("req.query") || trimmed.contains("filename") || trimmed.contains("filePath"))
                    && !trimmed.contains("path.resolve(")
                    && !trimmed.contains("root:")
                    && !content.contains("safePath")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-PATH-TRAVERSAL-SENDFILE".to_string(),
                        cve_id: "CWE-22 / SENDFILE-PATH-TRAVERSAL".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Arbitrary file read / sendFile with dynamic user path: vulnerable to '../' Path Traversal".to_string(),
                        transmitted_data: Some("untrusted_file_path".to_string()),
                        transmitted_type: Some("string (Unbounded File System Path)".to_string()),
                        remediation: "Pass { root: PUBLIC_DIR, dotfiles: 'deny' } to res.sendFile, or verify resolvedPath.startsWith(BASE_DIR).".to_string(),
                    });
                }

                // CVE-JS-REGEX-DOS-DYNAMIC-INPUT (CWE-1333 / Dynamic RegExp ReDoS)
                if trimmed.contains("new RegExp(")
                    && (trimmed.contains("req.query") || trimmed.contains("req.body") || trimmed.contains("pattern") || trimmed.contains("userInput"))
                    && !content.contains("escapeRegExp")
                    && !content.contains("lodash.escapeRegExp")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-REGEX-DOS-DYNAMIC-INPUT".to_string(),
                        cve_id: "CWE-1333 / DYNAMIC-REGEXP-CATASTROPHIC-REDOS".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Dynamically constructed RegExp from untrusted input without escaping: vulnerable to Catastrophic Backtracking ReDoS".to_string(),
                        transmitted_data: Some("dynamic_user_regex_pattern".to_string()),
                        transmitted_type: Some("string (Unescaped Regular Expression Pattern)".to_string()),
                        remediation: "Escape regex special characters with lodash.escapeRegExp or use linear matching.".to_string(),
                    });
                }

                // CVE-JS-CORS-WILDCARD-CREDENTIALS (CWE-942 / CORS Reflected Origin with Credentials)
                if (trimmed.contains("credentials: true") || trimmed.contains("Access-Control-Allow-Credentials', 'true'"))
                    && (trimmed.contains("origin: true") || trimmed.contains("origin: '*'") || trimmed.contains("req.headers.origin") || trimmed.contains("origin: req.header('origin')"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-CORS-WILDCARD-CREDENTIALS".to_string(),
                        cve_id: "CWE-942 / CORS-REFLECTED-ORIGIN-WITH-CREDENTIALS".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Overly permissive CORS: credentials: true with origin: true / reflected origin allows cross-origin credential theft".to_string(),
                        transmitted_data: Some("cors_credentials_configuration".to_string()),
                        transmitted_type: Some("CorsOptions (Permissive Wildcard + Credentials)".to_string()),
                        remediation: "Configure CORS with an explicit allowlist of authorized origins instead of reflecting client origin.".to_string(),
                    });
                }

                // CVE-JS-UNHANDLED-PROMISE-CRASH (CWE-755 / Async Route Unhandled Rejection)
                if (trimmed.contains("app.get(") || trimmed.contains("app.post(") || trimmed.contains("router.get(") || trimmed.contains("router.post("))
                    && trimmed.contains("async (req, res")
                    && !content.contains("try {")
                    && !content.contains("next(")
                    && !content.contains("express-async-errors")
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-UNHANDLED-PROMISE-CRASH".to_string(),
                        cve_id: "CWE-755 / ASYNC-ROUTE-UNHANDLED-REJECTION".to_string(),
                        severity: "HIGH".to_string(),
                        message: "Async Express route handler without try/catch or next(err): uncaught promise rejection crashes the Node.js process".to_string(),
                        transmitted_data: Some("unhandled_async_route_promise".to_string()),
                        transmitted_type: Some("AsyncFunction (Uncaught Rejected Promise)".to_string()),
                        remediation: "Wrap async route handler body in try/catch calling next(err), or use express-async-errors.".to_string(),
                    });
                }

                // CVE-JS-V8-TYPEDARRAY-SHARED-AB-RACE-OOB (V8-INTERNALS-CRITICAL / V8 Heap Bleed & SharedArrayBuffer TOCTOU)
                if ((trimmed.contains("Buffer.allocUnsafe(") || trimmed.contains("Buffer.allocUnsafeSlow("))
                    && !trimmed.contains(".fill(0)")
                    && !content.contains(".fill(0)"))
                    || (trimmed.contains("SharedArrayBuffer") && !content.contains("Atomics.load"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-JS-V8-TYPEDARRAY-SHARED-AB-RACE-OOB".to_string(),
                        cve_id: "V8-INTERNALS-CRITICAL / SHAREDARRAYBUFFER-TOCTOU-AND-ALLOCUNSAFE-BLEED".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "V8 uninitialized heap memory allocation (Buffer.allocUnsafe without .fill(0)) or SharedArrayBuffer access without Atomics.load: leaks residual private keys/JWTs from V8 heap (Nodebleed) or allows multi-worker TOCTOU bounds bypass".to_string(),
                        transmitted_data: Some("v8_uninitialized_or_shared_backing_store".to_string()),
                        transmitted_type: Some("Buffer / SharedArrayBuffer (Raw V8 Heap Memory)".to_string()),
                        remediation: "Use Buffer.alloc(size) (zero-filled by default) and read SharedArrayBuffer headers via Atomics.load() before copying into a private ArrayBuffer.".to_string(),
                    });
                }
            }

            // =========================================================
            // ⚙️ RÈGLES SPÉCIFIQUES C / C++ & FFI NATIF
            // =========================================================
            if is_cpp {
                // CVE-CPP-UNBOUNDED-COPY-UAF (CWE-120 / Classic Buffer Overflow in Native C/C++)
                if trimmed.contains("strcpy(")
                    || trimmed.contains("strcat(")
                    || trimmed.contains("sprintf(")
                    || trimmed.contains("gets(")
                    || (trimmed.contains("memcpy(") && !content.contains("sizeof(") && !content.contains("MAX_"))
                {
                    violations.push(LinterViolation {
                        file_path: file_path.to_string(),
                        line_number: line_no,
                        rule_id: "CVE-CPP-UNBOUNDED-COPY-UAF".to_string(),
                        cve_id: "CWE-120 / BUFFER-OVERFLOW-FFI".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: "Unbounded memory copy (strcpy/strcat/sprintf/gets/memcpy) in native C/C++ module leads to stack/heap buffer overflow and RCE".to_string(),
                        transmitted_data: Some("raw_c_buffer_pointer".to_string()),
                        transmitted_type: Some("char* / void* (Unbounded Memory Copy)".to_string()),
                        remediation: "Replace with bounds-checked snprintf(buf, sizeof(buf), ...), strncpy_s, or std::string / std::span.".to_string(),
                    });
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{ReadmeContract, ReadmeFrontmatter, SecurityRules};

    fn assert_rule_triggered(file_path: &str, code: &str, contract: Option<&ReadmeContract>, expected_rule: &str) {
        let violations = SecurityLinter::audit_file(file_path, code, contract);
        assert!(
            violations.iter().any(|v| v.rule_id == expected_rule),
            "Expected rule '{}' to trigger on file '{}', got violations: {:?}",
            expected_rule,
            file_path,
            violations
        );
    }

    #[test]
    fn test_all_7_brand_new_per_stack_cves() {
        // 1. Rust Async Starvation: CVE-RUST-BLOCKING-IN-ASYNC
        assert_rule_triggered(
            "server/api/websocket.rs",
            "pub async fn load_cfg() { let data = std::fs::read_to_string(\"cfg.json\").unwrap(); }",
            None,
            "CVE-RUST-BLOCKING-IN-ASYNC",
        );

        // 2. Web3 Signature Bypass: CVE-WEB3-AUTH-BYPASS-MOCK
        assert_rule_triggered(
            "server/api/auth_controller.mjs",
            "if (signature.startsWith('0xmock')) { return true; }",
            None,
            "CVE-WEB3-AUTH-BYPASS-MOCK",
        );

        // 3. MMO API IDOR: CVE-MMO-IDOR-TRUSTED-USERID
        assert_rule_triggered(
            "server/api/server.mjs",
            "const { userId, targetX, targetY } = JSON.parse(body);",
            None,
            "CVE-MMO-IDOR-TRUSTED-USERID",
        );

        // 4. Redis O(N) KEYS & Unauth Socket: CVE-DB-REDIS-UNAUTH-KEYS-DOS
        assert_rule_triggered(
            "data-nodes/cache-worker/redis_server.mjs",
            "if (cmd === 'KEYS') { return Object.keys(store); }",
            None,
            "CVE-DB-REDIS-UNAUTH-KEYS-DOS",
        );

        // 5. Python Path Traversal / ZipSlip: CVE-PY-PATH-TRAVERSAL-ARCHIVE
        assert_rule_triggered(
            "server/api/upload.py",
            "def unpack(zip_file):\n    zip_file.extractall('/tmp/uploads')",
            None,
            "CVE-PY-PATH-TRAVERSAL-ARCHIVE",
        );

        // 6. Node.js EventLoop Stream / ReDoS: CVE-JS-EVENTLOOP-REDOS-JSON
        assert_rule_triggered(
            "server/api/server.mjs",
            "req.on('data', chunk => { body += chunk.toString(); });",
            None,
            "CVE-JS-EVENTLOOP-REDOS-JSON",
        );

        // 7. C/C++ Buffer Overflow: CVE-CPP-UNBOUNDED-COPY-UAF
        assert_rule_triggered(
            "native/ffi/packet_codec.cpp",
            "void copy_tag(char* dst, const char* src) { strcpy(dst, src); }",
            None,
            "CVE-CPP-UNBOUNDED-COPY-UAF",
        );
    }

    #[test]
    fn test_all_5_new_rust_network_cves() {
        // 1. CVE-RUST-DESER-BOMB-OOM
        assert_rule_triggered(
            "server/api/websocket.rs",
            "pub fn decode(raw: &[u8]) { let pkt: Packet = bincode::deserialize(raw).unwrap(); }",
            None,
            "CVE-RUST-DESER-BOMB-OOM",
        );

        // 2. CVE-RUST-CANCEL-SAFETY-LOSS
        assert_rule_triggered(
            "server/api/websocket.rs",
            "async fn loop_io(sock: &mut TcpStream) { tokio::select! { _ = sock.read_exact(&mut buf) => {} } }",
            None,
            "CVE-RUST-CANCEL-SAFETY-LOSS",
        );

        // 3. CVE-RUST-TIMING-SIDECHANNEL
        assert_rule_triggered(
            "data-nodes/cache-worker/cache_service.rs",
            "pub fn check(stored: &str, expected: &str) -> bool { stored == expected }",
            None,
            "CVE-RUST-TIMING-SIDECHANNEL",
        );

        // 4. CVE-NET-SLOWLORIS-KEEPALIVE-DOS
        assert_rule_triggered(
            "server/api/websocket.rs",
            "pub async fn ws_accept(stream: TcpStream) { let ws = accept_async(stream).await; }",
            None,
            "CVE-NET-SLOWLORIS-KEEPALIVE-DOS",
        );

        // 5. CVE-NET-SSRF-DNS-REBIND
        assert_rule_triggered(
            "server/application/webhook.rs",
            "pub async fn notify(user_url: &str) { let _ = reqwest::get(user_url).await; }",
            None,
            "CVE-NET-SSRF-DNS-REBIND",
        );
    }

    #[test]
    fn test_all_web3_mmo_sast_arch_js_py_cves() {
        // Web3 rules
        assert_rule_triggered(
            "server/wallet/trade.rs",
            "pub fn withdraw() { wallet.transfer(amount); }",
            None,
            "CVE-WEB3-REENTRANCY-UNCHECKED",
        );
        assert_rule_triggered(
            "server/auth/crypto.rs",
            "pub fn check_sig(msg: &[u8], sig: &[u8]) { verify_signature(msg, sig); }",
            None,
            "CVE-WEB3-SIG-MALLEABILITY",
        );
        assert_rule_triggered(
            "contracts/auth.rs",
            "pub struct AuthVerifySignatureRequest { pub wallet: String }",
            None,
            "CVE-WEB3-UNVALIDATED-CHAINID",
        );
        assert_rule_triggered(
            "server/auth/session.rs",
            "pub fn login() {\n let session_token = issue();\n consume_nonce(nonce);\n}",
            None,
            "CVE-WEB3-NONCE-RACE",
        );

        // MMO & Concurrency
        assert_rule_triggered(
            "server/world/room.rs",
            "async fn tick(m: std::sync::Mutex<u32>) { foo().await; }",
            None,
            "CVE-RUST-ASYNC-MUTEX",
        );
        assert_rule_triggered(
            "server/api/websocket.rs",
            "pub fn init() { let (tx, rx) = tokio::sync::mpsc::unbounded_channel(); }",
            None,
            "CVE-RUST-UNBOUNDED-CHANNEL",
        );
        assert_rule_triggered(
            "server/domain/world.rs",
            "pub fn move_p(current: f32, delta: f32) -> f32 { current + delta }",
            None,
            "CVE-MMO-UNCHECKED-SPEEDHACK",
        );
        assert_rule_triggered(
            "server/wallet/inventory.rs",
            "pub fn buy(balance: &mut u64, cost: u64) { balance -= cost; }",
            None,
            "CVE-MATH-UNCHECKED-BALANCE",
        );

        // SAST & Injection
        assert_rule_triggered(
            "server/db/repo.rs",
            "let q = format!(\"SELECT * FROM users WHERE name = '{}'\", user);",
            None,
            "SAST_RAW_SQL_INJECTION",
        );
        assert_rule_triggered(
            "server/config.rs",
            "const API_KEY: &str = \"sk_live_9876543210abcdef\";\nlet api_key = \"sk_live_9876543210abcdef\";",
            None,
            "SAST_HARDCODED_SECRET",
        );
        assert_rule_triggered(
            "server/mem.rs",
            "pub fn raw() { unsafe { do_ptr(); } }",
            None,
            "CVE-RUST-UNSAFE-MEM",
        );
        assert_rule_triggered(
            "server/api/handler.rs",
            "pub fn handle(x: Option<u32>) { let v = x.unwrap(); }",
            None,
            "CVE-RUST-PANIC-DOS",
        );
        assert_rule_triggered(
            "server/sys.rs",
            "pub fn run(arg: &str) { std::process::Command::new(arg); }",
            None,
            "CVE-RUST-CMD-INJ",
        );

        // Python
        assert_rule_triggered(
            "server/worker.py",
            "data = pickle.loads(raw_bytes)",
            None,
            "CVE-PY-DESERIALIZE-RCE",
        );
        assert_rule_triggered(
            "server/shell.py",
            "os.system(user_cmd)",
            None,
            "CVE-PY-SHELL-INJ",
        );
        assert_rule_triggered(
            "server/fetch.py",
            "res = requests.get(target_url)",
            None,
            "CVE-PY-SSRF-UNVALIDATED",
        );
        assert_rule_triggered(
            "server/api/routes.py",
            "def create_item(payload: dict):",
            None,
            "CVE-PY-UNTYPED-PAYLOAD",
        );

        // Architecture
        let contract = ReadmeContract {
            folder_path: "server/domain".to_string(),
            frontmatter: ReadmeFrontmatter {
                id: "domain".to_string(),
                layer: "domain".to_string(),
                max_lines_per_file: 2,
                allowed_imports: vec![],
                forbidden_imports: vec!["sqlx".to_string()],
                security_rules: SecurityRules {
                    no_raw_sql: true,
                    no_env_access: true,
                    untrusted_client_input: false,
                    deterministic_only: true,
                },
            },
            ears_requirements: vec![],
            exported_signatures: vec![],
            blake3_hash: "hash".to_string(),
        };
        assert_rule_triggered(
            "server/domain/logic.rs",
            "use sqlx::PgPool;\nfn a() {}\nfn b() {}",
            Some(&contract),
            "RULE_FORBIDDEN_IMPORT",
        );
        assert_rule_triggered(
            "server/domain/logic.rs",
            "fn a() {}\nfn b() {}\nfn c() {}",
            Some(&contract),
            "RULE_MAX_LINES",
        );
        assert_rule_triggered(
            "server/domain/logic.rs",
            "pub fn key() { let k = std::env::var(\"SECRET\"); }",
            None,
            "SAST_DIRECT_ENV_ACCESS",
        );

        // JS / TS Front-End
        assert_rule_triggered(
            "client/ui/view.js",
            "el.innerHTML = userBio;",
            None,
            "CVE-JS-XSS-DOM",
        );
        assert_rule_triggered(
            "server/api/controller.ts",
            "export function update(payload: any) {}",
            None,
            "CVE-JS-PROTO-POLLUTION",
        );
        assert_rule_triggered(
            "server/api/exec.js",
            "child_process.exec(cmd);",
            None,
            "CVE-JS-CMD-INJ",
        );
        assert_rule_triggered(
            "client/ui/eval.js",
            "const res = eval(dynamicCode);",
            None,
            "CVE-JS-CODE-INJ",
        );
        assert_rule_triggered(
            "client/ui/canvas.ts",
            "const tex = await PIXI.Assets.load(avatar_url);",
            None,
            "CVE-CLIENT-UNSAFE-ASSETS",
        );
    }

    #[test]
    fn test_all_9_new_web3_cves() {
        // 1. Flash Loan Oracle Manipulation
        assert_rule_triggered(
            "contracts/lending.sol",
            "function getPrice() public view returns (uint) { (uint112 r0, uint112 r1,) = pair.getReserves(); return r0 / r1; }",
            None,
            "CVE-WEB3-ORACLE-SPOT-MANIPULATION",
        );

        // 2. tx.origin Phishing
        assert_rule_triggered(
            "contracts/vault.sol",
            "function withdraw() public { require(tx.origin == owner); }",
            None,
            "CVE-WEB3-TX-ORIGIN-PHISHING",
        );

        // 3. Unchecked Transfer Return
        assert_rule_triggered(
            "contracts/vault.sol",
            "function deposit(uint amount) public { token.transfer(msg.sender, amount); }",
            None,
            "CVE-WEB3-UNCHECKED-TRANSFER-RETURN",
        );

        // 4. Permit Front-Running
        assert_rule_triggered(
            "contracts/vault.sol",
            "function depositWithPermit() public { token.permit(owner, spender, value, deadline, v, r, s); token.transferFrom(owner, address(this), value); }",
            None,
            "CVE-WEB3-PERMIT-FRONT-RUNNING",
        );

        // 5. Hook Reentrancy (ERC-777 / ERC-1363)
        assert_rule_triggered(
            "contracts/pool.sol",
            "function tokensReceived(address operator, address from, address to, uint256 amount, bytes calldata data, bytes calldata operatorData) external { update(); }",
            None,
            "CVE-WEB3-HOOK-REENTRANCY",
        );

        // 6. Account-Model Missing Owner Check
        assert_rule_triggered(
            "programs/vault/src/lib.rs",
            "pub fn process(account: &AccountInfo) { let val = account.data.borrow(); }",
            None,
            "CVE-VM-ACCOUNT-MISSING-OWNER",
        );

        // 7. Read-Only Reentrancy on Curve / Balancer LP
        assert_rule_triggered(
            "contracts/oracle.sol",
            "function getLpRate() public view returns (uint) { return curvePool.get_virtual_price(); }",
            None,
            "CVE-WEB3-READONLY-REENTRANCY",
        );

        // 8. Dirty Memory in Inline Assembly
        assert_rule_triggered(
            "contracts/hasher.sol",
            "function isClean(bool flag) public pure { assembly { mstore(0x00, flag) } }",
            None,
            "CVE-WEB3-DIRTY-MEMORY-ASSEMBLY",
        );

        // 9. ERC-4626 Inflation Attack
        assert_rule_triggered(
            "contracts/vault.sol",
            "contract MyVault is ERC4626 { function convertToShares(uint assets) public view returns (uint) { return assets * totalSupply / totalAssets; } }",
            None,
            "CVE-WEB3-ERC4626-INFLATION-ATTACK",
        );
    }

    #[test]
    fn test_all_4_modular_vault_web3_cves() {
        // 1. Vault Rounding Direction
        assert_rule_triggered(
            "contracts/vault.sol",
            "function convertToShares(uint assets) public view returns (uint) { return assets * 1e18 / totalAssets; }",
            None,
            "CVE-WEB3-VAULT-ROUNDING-DIRECTION",
        );

        // 2. Stale Oracle Heartbeat
        assert_rule_triggered(
            "contracts/oracle.sol",
            "function getPrice() public view returns (int) { (, int price, , , ) = priceFeed.latestRoundData(); return price; }",
            None,
            "CVE-WEB3-STALE-ORACLE-HEARTBEAT",
        );

        // 3. Unprotected Flash Loan Callback
        assert_rule_triggered(
            "contracts/flash.sol",
            "function onFlashLoan(address initiator, address token, uint256 amount, uint256 fee, bytes calldata data) external returns (bytes32) { return keccak256('ERC3156FlashBorrower.onFlashLoan'); }",
            None,
            "CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR",
        );

        // 4. Missing Signature Deadline
        assert_rule_triggered(
            "contracts/signer.sol",
            "function permit(address owner, address spender, uint value, uint8 v, bytes32 r, bytes32 s) public { bytes32 hash = keccak256(abi.encode(owner, spender, value)); address signer = ecrecover(hash, v, r, s); }",
            None,
            "CVE-WEB3-SIGNATURE-DEADLINE-MISSING",
        );
    }

    #[test]
    fn test_false_positive_elimination_smart_contracts() {
        // 1. Interface declaration ending with ';' should NOT trigger CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR
        let iface_code = "interface IFlashLoan { function onFlashLoan(address initiator, address token, uint256 amount, uint256 fee, bytes calldata data) external returns (bytes32); }";
        let violations = SecurityLinter::audit_file("src/interfaces/IFlashLoan.sol", iface_code, None);
        assert!(violations.iter().all(|v| v.rule_id != "CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR"), "Interface declaration should not trigger callback selector vulnerability");

        // 2. Unit test files (.t.sol or inside test/) should NOT trigger contract vulnerabilities
        let test_code = "contract VaultTest is Test { function testDeposit() public { token.transfer(alice, 100); } }";
        let test_violations = SecurityLinter::audit_file("test/unit/Vault.t.sol", test_code, None);
        assert!(test_violations.is_empty(), "Unit test files must be excluded from production vulnerability alerts");

        // 3. Certora runner python scripts should NOT trigger shell injection alerts
        let certora_py = "import subprocess\nsubprocess.run(f'certoraRun {conf}', shell=True)";
        let py_violations = SecurityLinter::audit_file("certora/scripts/runHealth.py", certora_py, None);
        assert!(py_violations.is_empty(), "Certora formal verification scripts must not be flagged as production shell injection");

        // 4. Overriding transfer method with super.transfer should NOT trigger CVE-WEB3-UNCHECKED-TRANSFER-RETURN
        let vault_override = "contract EVault { function transfer(address to, uint256 amount) public virtual override returns (bool) { return super.transfer(to, amount); } }";
        let override_violations = SecurityLinter::audit_file("src/EVault/EVault.sol", vault_override, None);
        assert!(override_violations.iter().all(|v| v.rule_id != "CVE-WEB3-UNCHECKED-TRANSFER-RETURN"), "super.transfer ERC20 override must not be flagged");

        // 5. Tests and Interfaces should NEVER trigger RULE_MAX_LINES
        let big_code = "line\n".repeat(800);
        let contract = ReadmeContract::parse("root", "---\nid: root\nlayer: domain\nmax_lines_per_file: 300\n---").unwrap();
        let test_lines_v = SecurityLinter::audit_file("test/unit/BigTest.t.sol", &big_code, Some(&contract));
        assert!(test_lines_v.iter().all(|v| v.rule_id != "RULE_MAX_LINES"), "Tests must not trigger RULE_MAX_LINES");
        let iface_lines_v = SecurityLinter::audit_file("src/EVault/IEVault.sol", &big_code, Some(&contract));
        assert!(iface_lines_v.iter().all(|v| v.rule_id != "RULE_MAX_LINES"), "Interfaces must not trigger RULE_MAX_LINES");
    }

    #[test]
    fn test_3_high_yield_defi_fork_cves() {
        // 1. Hook Deferred Checks Desync
        assert_rule_triggered(
            "contracts/CustomHookTarget.sol",
            "contract CustomHookTarget is IHookTarget { function checkVaultStatus() external { otherVault.call(abi.encodeWithSignature('withdraw()')); } }",
            None,
            "CVE-WEB3-HOOK-DEFERRED-CHECKS-DESYNC",
        );

        // 2. Oracle Decimal Scaling Truncation
        assert_rule_triggered(
            "contracts/oracle/PriceFeed.sol",
            "function getNormalizedPrice(uint price, uint8 decimals) public pure returns (uint) { return price * 10 ** (18 - decimals); }",
            None,
            "CVE-WEB3-DECIMAL-SCALING-TRUNCATION",
        );

        // 3. Fork Reward Staking Dilution
        assert_rule_triggered(
            "contracts/staking/StakingForkVault.sol",
            "contract StakingVault { function deposit(uint amount) external { rewardToken.transfer(msg.sender, 100); } }",
            None,
            "CVE-WEB3-FORK-REWARD-STAKING-DILUTION",
        );
    }

    #[test]
    fn test_advanced_defi_and_evc_rules() {
        // 1. DEFI-ASYMMETRIC-ROUNDING (Solidity)
        let vault_code = "contract CustomVault { function convertToShares(uint assets) public view returns (uint) { return assets * 10 / totalAssets; } }";
        let v1 = SecurityLinter::audit_file("contracts/vault/CustomVault.sol", vault_code, None);
        assert!(v1.iter().any(|v| v.rule_id == "DEFI-ASYMMETRIC-ROUNDING"), "Should detect asymmetric rounding in vault");

        // 2. DEFI-ASYMMETRIC-ROUNDING (Rust Economy)
        let economy_code = "pub fn compute_fee(balance: u64) -> u64 { let fee = balance / 100; fee }";
        let v2 = SecurityLinter::audit_file("src/economy.rs", economy_code, None);
        assert!(v2.iter().any(|v| v.rule_id == "DEFI-ASYMMETRIC-ROUNDING"), "Should detect unrounded economy tax division");

        // 3. DEFI-STALE-CACHE-AFTER-CALL (VaultCache TOCTOU)
        let stale_cache_code = "function withdraw(uint amount) public { VaultCache memory vaultCache = loadVaultCache(); target.call(abi.encodeWithSignature('hook()')); uint bal = vaultCache.balance; }";
        let v3 = SecurityLinter::audit_file("src/vault/Manager.sol", stale_cache_code, None);
        assert!(v3.iter().any(|v| v.rule_id == "DEFI-STALE-CACHE-AFTER-CALL"), "Should detect stale cache reuse after external call");

        // 4. EVC-HOOK-BYPASS
        let hook_bypass_code = "contract MyHook { function checkVaultStatus() external { token.transfer(msg.sender, 100); } }";
        let v4 = SecurityLinter::audit_file("src/hooks/MyHook.sol", hook_bypass_code, None);
        assert!(v4.iter().any(|v| v.rule_id == "EVC-HOOK-BYPASS"), "Should detect unverified EVC hook bypass");
    }

    #[test]
    fn test_3_ultra_critical_domain_logic_rules() {
        // 1. DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER
        let min_max_code = "contract ChainlinkPriceOracle { function getPrice() external view returns (uint256) { (, int256 answer, , , ) = feed.latestRoundData(); require(answer > 0); return uint256(answer); } }";
        let v1 = SecurityLinter::audit_file("src/adapter/oracle/ChainlinkPriceOracle.sol", min_max_code, None);
        assert!(v1.iter().any(|v| v.rule_id == "DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER"), "Should detect missing minAnswer/maxAnswer circuit breaker");

        // Safe code with minAnswer
        let safe_min_max = "contract SafeOracle { function getPrice() external view returns (uint256) { (, int256 answer, , , ) = feed.latestRoundData(); require(answer > minAnswer && answer < maxAnswer); return uint256(answer); } }";
        let v1_safe = SecurityLinter::audit_file("src/adapter/oracle/SafeOracle.sol", safe_min_max, None);
        assert!(v1_safe.iter().all(|v| v.rule_id != "DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER"), "Safe oracle should not trigger circuit breaker rule");

        // 2. DEFI-ORACLE-L2-SEQUENCER-RACE
        let l2_oracle_code = "contract ArbitrumOracle { function getPrice() external view returns (uint) { (, int256 answer, , , ) = feed.latestRoundData(); return uint(answer); } }";
        let v2 = SecurityLinter::audit_file("src/adapter/arbitrum/ArbitrumOracle.sol", l2_oracle_code, None);
        assert!(v2.iter().any(|v| v.rule_id == "DEFI-ORACLE-L2-SEQUENCER-RACE"), "Should detect missing L2 sequencer uptime feed on Arbitrum");

        // Safe L2 code with sequencer uptime and grace period
        let safe_l2_code = "contract SafeArbitrumOracle { function getPrice() external view returns (uint) { require(isSequencerActive && block.timestamp - startedAt > GRACE_PERIOD); (, int256 answer, , , ) = feed.latestRoundData(); return uint(answer); } }";
        let v2_safe = SecurityLinter::audit_file("src/adapter/arbitrum/SafeArbitrumOracle.sol", safe_l2_code, None);
        assert!(v2_safe.iter().all(|v| v.rule_id != "DEFI-ORACLE-L2-SEQUENCER-RACE"), "Safe L2 oracle should not trigger sequencer race rule");

        // 3. DEFI-TWAP-SHORT-WINDOW
        let short_twap_code = "contract UniswapOracle { function getTwap() external view returns (uint) { uint32[] memory secondsAgo = new uint32[](2); secondsAgo[0] = 300; secondsAgo[1] = 0; pool.observe(secondsAgo); } }";
        let v3 = SecurityLinter::audit_file("src/adapter/uniswap/UniswapOracle.sol", short_twap_code, None);
        assert!(v3.iter().any(|v| v.rule_id == "DEFI-TWAP-SHORT-WINDOW"), "Should detect manipulable short TWAP window (< 30 min)");
    }

    #[test]
    fn test_all_18_new_massive_polyglot_cves() {
        // --- 6 Python CVEs ---
        // 1. Jinja2 SSTI
        assert_rule_triggered(
            "src/server/views.py",
            "import jinja2\ndef render(name):\n    return render_template_string(f'Hello {name}')",
            None,
            "CVE-PY-JINJA2-SSTI-INJECTION",
        );
        // 2. Unsafe YAML Load
        assert_rule_triggered(
            "src/config/loader.py",
            "import yaml\ndef load_conf(s):\n    return yaml.load(s)",
            None,
            "CVE-PY-YAML-UNSAFE-LOAD",
        );
        // 3. SQLAlchemy Raw Text SQL Injection
        assert_rule_triggered(
            "src/db/repo.py",
            "from sqlalchemy import text\ndef get_user(uid):\n    return session.execute(text(f'SELECT * FROM users WHERE id = {uid}'))",
            None,
            "CVE-PY-SQL-ALCHEMY-TEXT-INJECTION",
        );
        // 4. JWT none algorithm
        assert_rule_triggered(
            "src/auth/token.py",
            "import jwt\ndef decode_tok(t):\n    return jwt.decode(t, options={'verify_signature': False})",
            None,
            "CVE-PY-JWT-NONE-ALGORITHM",
        );
        // 5. Paramiko AutoAddPolicy MITM
        assert_rule_triggered(
            "src/ssh/client.py",
            "import paramiko\nclient = paramiko.SSHClient()\nclient.set_missing_host_key_policy(paramiko.AutoAddPolicy())",
            None,
            "CVE-PY-PARAMIKO-MISSING-HOST-KEY",
        );
        // 6. XML XXE Entity Expansion
        assert_rule_triggered(
            "src/xml_parser.py",
            "import xml.etree.ElementTree as ET\ndef parse(data):\n    return ET.fromstring(data)",
            None,
            "CVE-PY-XML-XXE-ENTITY-EXPANSION",
        );

        // --- 6 Rust CVEs ---
        // 1. Integer Overflow Cast Truncation
        assert_rule_triggered(
            "src/network/codec.rs",
            "pub fn parse_frame(wire_len: u64) -> u32 { let size = wire_len as u32; size }",
            None,
            "CVE-RUST-INTEGER-OVERFLOW-CAST",
        );
        // 2. Transmute Size Mismatch
        assert_rule_triggered(
            "src/mem/cast.rs",
            "pub fn unsafe_cast(x: u64) -> [u8; 8] { unsafe { std::mem::transmute(x) } }",
            None,
            "CVE-RUST-TRANSMUTE-SIZE-MISMATCH",
        );
        // 3. Forgotten Mem Leak / Deadlock
        assert_rule_triggered(
            "src/sync/leak.rs",
            "pub fn bypass_drop(guard: std::sync::MutexGuard<u32>) { std::mem::forget(guard); }",
            None,
            "CVE-RUST-FORGOTTEN-MEM-LEAK",
        );
        // 4. Unchecked Index Slicing Panic DOS
        assert_rule_triggered(
            "src/packet/reader.rs",
            "pub fn read_packet(packet: &[u8], offset: usize, len: usize) -> &[u8] { &packet[offset..offset + len] }",
            None,
            "CVE-RUST-UNCHECKED-INDEX-SLICING",
        );
        // 5. Thread Poison Ignored
        assert_rule_triggered(
            "src/state/shared.rs",
            "pub fn get_val(m: &std::sync::Mutex<u32>) -> u32 { *m.lock().unwrap_or_else(|e| e.into_inner()) }",
            None,
            "CVE-RUST-THREAD-POISON-IGNORED",
        );
        // 6. FFI Unchecked Null Pointer Dereference
        assert_rule_triggered(
            "src/ffi/bridge.rs",
            "pub unsafe extern \"C\" fn handle(ptr: *const u8) -> u8 { unsafe { *ptr } }",
            None,
            "CVE-RUST-FFI-UNCHECKED-NULL-PTR",
        );

        // --- 6 Node.js / TS CVEs ---
        // 1. Node.js VM Sandbox Escape
        assert_rule_triggered(
            "src/sandbox/runner.js",
            "const vm = require('vm'); function run(code) { return vm.runInNewContext(code); }",
            None,
            "CVE-JS-VM-SANDBOX-ESCAPE",
        );
        // 2. Outbound SSRF via Axios/Fetch
        assert_rule_triggered(
            "src/controllers/proxy.js",
            "async function forward(req, res) { const data = await axios.get(req.query.url); }",
            None,
            "CVE-JS-SSRF-DYNAMIC-AXIOS-FETCH",
        );
        // 3. SendFile Path Traversal
        assert_rule_triggered(
            "src/routes/files.js",
            "app.get('/download', (req, res) => { res.sendFile(req.query.filename); });",
            None,
            "CVE-JS-PATH-TRAVERSAL-SENDFILE",
        );
        // 4. Dynamic RegExp ReDoS
        assert_rule_triggered(
            "src/search/query.js",
            "function search(req) { const rx = new RegExp(req.query.pattern); }",
            None,
            "CVE-JS-REGEX-DOS-DYNAMIC-INPUT",
        );
        // 5. Permissive CORS with Credentials
        assert_rule_triggered(
            "src/app.js",
            "app.use(cors({ origin: true, credentials: true }));",
            None,
            "CVE-JS-CORS-WILDCARD-CREDENTIALS",
        );
        // 6. Async Route Unhandled Promise Crash
        assert_rule_triggered(
            "src/routes/api.js",
            "app.get('/data', async (req, res) => { const user = await db.fetch(); res.json(user); });",
            None,
            "CVE-JS-UNHANDLED-PROMISE-CRASH",
        );
    }

    #[test]
    fn test_4_novel_ultra_critical_web3_engineering_cves() {
        // 1. EIP-1153 Transient Storage Composability Leak (Dirty TSTORE)
        assert_rule_triggered(
            "contracts/v4/FlashAccountingHook.sol",
            "contract FlashHook { function unlockCallback() external { assembly { tstore(0x01, caller()) } } }",
            None,
            "CVE-WEB3-EIP1153-TRANSIENT-COMPOSABILITY-LEAK",
        );

        // 2. EIP-7702 EOA Code Bypass & Slot-0 Collision
        assert_rule_triggered(
            "contracts/launchpad/FairMint.sol",
            "contract FairMint { function mint() external { require(msg.sender.code.length == 0, 'Only EOA'); } }",
            None,
            "CVE-WEB3-EIP7702-EOA-CODE-BYPASS-COLLISION",
        );

        // 3. ERC-4337 Paymaster Gas Siphon via postOp Griefing
        assert_rule_triggered(
            "contracts/aa/TokenPaymaster.sol",
            "contract TokenPaymaster { function _postOp(uint8 mode, bytes calldata context, uint256 actualGasCost) internal { feeToken.transferFrom(user, address(this), actualGasCost); } }",
            None,
            "CVE-WEB3-ERC4337-PAYMASTER-GAS-SIPHON",
        );

        // 4. Omnichain Cross-Chain Channel Bricking & Burned Funds Loss
        assert_rule_triggered(
            "contracts/bridge/OmnichainOFT.sol",
            "contract OmnichainOFT { function _lzReceive(uint16 srcChainId, bytes memory payload) internal { (address to, uint amount) = abi.decode(payload, (address, uint)); usdc.safeTransfer(to, amount); } }",
            None,
            "CVE-WEB3-OMNICHAIN-LZ-CHANNEL-BRICKING",
        );
    }

    #[test]
    fn test_6_masterclass_unreferenced_deep_engineering_cves() {
        // 1. EIP-150 63/64th Gas Stipend Relayer OOG Griefing
        assert_rule_triggered(
            "contracts/relayer/MetaTxForwarder.sol",
            "contract Forwarder { function execute(address target, bytes calldata data, uint256 nonce) external { nonces[msg.sender] = nonce; (bool ok, ) = target.call(data); } }",
            None,
            "CVE-WEB3-EIP150-63-64-GAS-STIPEND-GRIEFING",
        );

        // 2. ZK-SNARK BN254 Scalar Field Modulo Aliasing (5x Spend)
        assert_rule_triggered(
            "contracts/zk/Groth16Pool.sol",
            "contract Groth16Pool { function withdraw(uint256[8] calldata proof, uint256 nullifierHash) external { require(!nullifiers[nullifierHash]); verifier.verifyProof(proof, [nullifierHash]); } }",
            None,
            "CVE-WEB3-ZK-SNARK-FIELD-OVERFLOW-ALIASING",
        );

        // 3. Merkle Tree 64-Byte Leaf Second Preimage Collision
        assert_rule_triggered(
            "contracts/airdrop/MerkleDistributor.sol",
            "contract MerkleDistributor { function claim(address account, uint256 amount, bytes32[] calldata proof) external { bytes32 leaf = keccak256(abi.encode(account, amount)); require(MerkleProof.verify(proof, merkleRoot, leaf)); } }",
            None,
            "CVE-WEB3-MERKLE-SECOND-PREIMAGE-LEAF-NODE-COLLISION",
        );

        // 4. CREATE2 Unsalted Sender Front-Run & Metamorphic Deployment
        assert_rule_triggered(
            "contracts/factory/VaultFactory.sol",
            "contract VaultFactory { function deploy(bytes32 userSalt) external returns (address) { return address(new Vault{salt: userSalt}()); } }",
            None,
            "CVE-WEB3-CREATE2-METAMORPHIC-REINIT-RUG",
        );

        // 5. Rust Async Pin Structural Unpin Soundness Break (Self-Referential UAF)
        assert_rule_triggered(
            "src/runtime/custom_future.rs",
            "pub fn poll_stream(mut self: std::pin::Pin<&mut Self>) { let this = unsafe { std::pin::Pin::get_unchecked_mut(self) }; }",
            None,
            "CVE-RUST-PIN-STRUCTURAL-UNPIN-SOUNDNESS-UB",
        );

        // 6. V8 Buffer.allocUnsafe Heap Bleed & SharedArrayBuffer TOCTOU Race
        assert_rule_triggered(
            "src/network/frame_encoder.js",
            "function allocateFrame(size) { const frame = Buffer.allocUnsafe(size); return frame; }",
            None,
            "CVE-JS-V8-TYPEDARRAY-SHARED-AB-RACE-OOB",
        );
    }

    #[test]
    fn test_6_l1_consensus_rules_and_fp_filters() {
        assert!(
            SecurityLinter::get_supported_cve_rules().len() >= 94,
            "Catalog should contain at least 94 security rules"
        );

        // 1. P2P & Consensus Flaws: Unbounded Borsh / Zstd Deserialization OOM Bomb
        assert_rule_triggered(
            "chain/network/src/peer/peer_actor.rs",
            "pub fn parse_message(raw_msg: &[u8]) { let msg = PeerMessage::try_from_slice(raw_msg).unwrap(); }",
            None,
            "CVE-L1-BORSH-UNBOUNDED-DESER-OOM",
        );

        // 2. Fee Payment Bypass & Contracts Execution Flows: Wasm Host Allocation Before Gas Prepayment
        assert_rule_triggered(
            "runtime/vm-runner/src/logic/logic.rs",
            "pub fn read_register(&mut self, len: u64) { let mut buf = vec![0u8; len as usize]; self.memory.read_memory(ptr, &mut buf); self.result_state.gas_counter.burn_gas(cost); }",
            None,
            "CVE-L1-RUNTIME-GAS-PREPAY-BYPASS",
        );

        // 3. Stealing/Loss of Funds & Balance Manipulation: Account Balance Mutation Without Storage Staking Check
        assert_rule_triggered(
            "runtime/runtime/src/actions.rs",
            "pub fn action_transfer(account: &mut Account, deposit: u128) { account.set_amount(account.amount().saturating_sub(deposit)); }",
            None,
            "CVE-L1-BALANCE-STORAGE-STAKING-DESYNC",
        );

        // 4. Consensus Flaws: Stateless Validation ChunkStateWitness Processed Before Signature Verification
        assert_rule_triggered(
            "chain/client/src/stateless_validation/chunk_validator.rs",
            "pub fn handle_chunk_state_witness(witness: SignedValidPeriodTransactions) { apply_chunk_state_witness(witness); save_latest_witness(&witness); }",
            None,
            "CVE-L1-CONSENSUS-UNVERIFIED-WITNESS-CHUNK",
        );

        // 5. Unauthorized Transaction & Transaction Manipulation: NEP-366 DelegateAction Cross-Shard Replay
        assert_rule_triggered(
            "runtime/runtime/src/verifier.rs",
            "pub fn verify_delegate_action(delegate_action: &SignedDelegateAction) -> bool { verify_action_signature(&delegate_action.signature, &delegate_action.delegate_action) }",
            None,
            "CVE-L1-CROSS-SHARD-RECEIPT-NONCE-REPLAY",
        );

        // 6. Cryptographic Flaws: Curve25519 / BLS12-381 Unchecked Point Decompression & Small-Subgroup Malleability
        assert_rule_triggered(
            "core/crypto/src/signature.rs",
            "pub fn decompress_pk(bytes: &[u8]) { let point = CompressedEdwardsY::from_slice(bytes).decompress_unchecked(); }",
            None,
            "CVE-L1-CRYPTO-BLIND-SCALAR-MERKLE-MALLEABILITY",
        );

        // 7. Anti-False-Positive Filter Verification:
        //    a) #[cfg(test)] block at bottom of production Rust file must NOT trigger alerts
        let prod_with_cfg_test = r#"
pub fn safe_prod_fn() -> u32 { 42 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_borsh() {
        let msg = PeerMessage::try_from_slice(raw_msg).unwrap();
        unsafe { *ptr = 1; }
    }
}
"#;
        let res_cfg_test = SecurityLinter::audit_file("chain/network/src/peer_manager.rs", prod_with_cfg_test, None);
        assert!(
            res_cfg_test.is_empty(),
            "Code inside #[cfg(test)] module must not trigger false-positive security alerts, got: {:?}",
            res_cfg_test
        );

        //    b) Audited unsafe block with `// SAFETY:` comment must NOT trigger CVE-RUST-UNSAFE-MEM
        let documented_unsafe = r#"
pub fn aligned_read(ptr: *const u64) -> u64 {
    // SAFETY: Caller guarantees ptr is non-null, aligned to 8 bytes, and valid for reads.
    unsafe { *ptr }
}
"#;
        let res_safety = SecurityLinter::audit_file("core/store/src/mmap.rs", documented_unsafe, None);
        assert!(
            !res_safety.iter().any(|a| a.rule_id == "CVE-RUST-UNSAFE-MEM"),
            "Audited unsafe block with // SAFETY: comment must not trigger CVE-RUST-UNSAFE-MEM"
        );
    }
}



