# 🪐 AETHER-ENGINE — CATALOGUE OFFICIEL DES 150+ RÈGLES CWE & PATTERNS ZERO-DAY
> **Moteur SAST Spatial AetherScope V2**  
> **Total des règles actives intégrées :** `152 règles d'audit profondes`  
> **Technologies auditées :** PHP (WordPress / API), Rust (Memory Unsafe / LLVM), C++ (Réseau / Protocoles), Node.js / TypeScript, Smart Contracts (Solidity / Move / ZK).

Ce catalogue recense l'ensemble des règles statiques, vérifications de contrats EARS et détecteurs de vulnérabilités embarqués dans le moteur AetherScope V2.

## 📊 Sommaire des Catégories

- [Web3 / Crypto (34 règles)](#web3--crypto)
- [PHP / Web Security (25 règles)](#php--web-security)
- [SAST / Injection (17 règles)](#sast--injection)
- [Rust / Network (14 règles)](#rust--network)
- [Front-End / DOM (13 règles)](#front-end--dom)
- [C / C++ / Network (10 règles)](#c--c++--network)
- [MMO / Concurrency (5 règles)](#mmo--concurrency)
- [DeFi / Oracles (4 règles)](#defi--oracles)
- [Architecture / Boundary (4 règles)](#architecture--boundary)
- [SAST / Injection & RCE (3 règles)](#sast--injection--rce)
- [DeFi / Invariants (3 règles)](#defi--invariants)
- [Cryptography / Zero-Knowledge (2 règles)](#cryptography--zero-knowledge)
- [Account-Model VM / Smart Contracts (2 règles)](#account-model-vm--smart-contracts)
- [Rust / Memory Safety & RCE (2 règles)](#rust--memory-safety--rce)
- [Rust / Memory Safety (1 règles)](#rust--memory-safety)
- [Rust / Concurrency (1 règles)](#rust--concurrency)
- [Wasm Contracts / Storage (1 règles)](#wasm-contracts--storage)
- [Layer 2 / Rollups (1 règles)](#layer-2--rollups)
- [Web3 / Account Abstraction (1 règles)](#web3--account-abstraction)
- [SAST / Deserialization (1 règles)](#sast--deserialization)
- [Move VM / Memory Safety & RCE (1 règles)](#move-vm--memory-safety--rce)
- [Rust / Compiler & Cryptography (1 règles)](#rust--compiler--cryptography)
- [Rust / Memory Safety & ABI (1 règles)](#rust--memory-safety--abi)
- [Rust / Allocator & Memory Safety (1 règles)](#rust--allocator--memory-safety)
- [Rust / ABI & FFI Soundness (1 règles)](#rust--abi--ffi-soundness)
- [Rust / Concurrency & LLVM Soundness (1 règles)](#rust--concurrency--llvm-soundness)
- [Rust / OS Kernel & Storage Soundness (1 règles)](#rust--os-kernel--storage-soundness)
- [C / C++ / FFI (1 règles)](#c--c++--ffi)

---

## 🛡️ Web3 / Crypto (34 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-L1-RUNTIME-GAS-PREPAY-BYPASS` | **L1-FEE-BYPASS-CRITICAL / HOST-FN-ALLOCATION-BEFORE-GAS-BURN** | Contournement de Facturation de Gaz Wasm (Allocation Mémoire / I/O Avant burn_gas) | Fonction hôte du runtime Wasm (VMLogic / host functions) exécutant une allocation mémoire dynamique (vec![0; len], read_memory) ou un accès ... |
| 🔴 **CRITICAL** | `CVE-L1-BALANCE-STORAGE-STAKING-DESYNC` | **L1-FUNDS-CRITICAL / BALANCE-MUTATION-WITHOUT-STORAGE-STAKE-CHECK** | Désynchronisation Balance / Storage Staking & Manipulation de Solde L1 (set_amount / saturating_sub) | Mutation directe du solde ou du stake d'un compte (set_amount, set_locked, saturating_sub sur Balance) sans safe_add_balance/checked_sub ou ... |
| 🔴 **CRITICAL** | `CVE-L1-CONSENSUS-UNVERIFIED-WITNESS-CHUNK` | **L1-CONSENSUS-CRITICAL / STATELESS-WITNESS-UNVERIFIED-ENDORSEMENT** | Traitement de ChunkStateWitness / Endorsement Avant Vérification Cryptographique du Validateur | Application ou mise en cache d'un ChunkStateWitness, PartialEncodedChunk ou Approval dans le consensus Nightshade/Doomslug avant la validati... |
| 🔴 **CRITICAL** | `CVE-L1-CROSS-SHARD-RECEIPT-NONCE-REPLAY` | **L1-TX-CRITICAL / DELEGATE-ACTION-ACCESS-KEY-NONCE-BYPASS** | Rejeu de Meta-Transaction NEP-366 (DelegateAction) & Contournement d'Allowance AccessKey | Exécution de DelegateAction ou SignedTransaction sans vérification stricte de max_block_height, de la portée FunctionCallPermission ou de la... |
| 🔴 **CRITICAL** | `CVE-L1-CRYPTO-BLIND-SCALAR-MERKLE-MALLEABILITY` | **L1-CRYPTO-CRITICAL / UNCHECKED-CURVE-DECOMPRESS-TORSION-SPLIT** | Divergence de Consensus Cryptographique (Point BLS12-381 / Ed25519 / BN254 Non-Canonique ou Torsion) | Décompression de point elliptique ou vérification de signature/preuve Merkle via des primitives _unchecked (from_bytes_unchecked, decompress... |
| 🔴 **CRITICAL** | `CVE-WEB3-REENTRANCY-UNCHECKED` | **CWE-841 / REENTRANCY** | Anti-Réentrance Web3 & MMO | Transfert d'actifs ou mise à jour de solde sans ReentrancyGuard ou Checks-Effects-Interactions. |
| 🔴 **CRITICAL** | `CVE-WEB3-SIG-MALLEABILITY` | **CWE-347 / ECDSA-MALLEABILITY** | Malléabilité de Signature ECDSA (secp256k1) | Vérification de signature sans validation du s-value borné (s < secp256k1_n/2), ouvrant au rejeu. |
| 🔴 **CRITICAL** | `CVE-WEB3-UNVALIDATED-CHAINID` | **CWE-294 / CROSS-CHAIN-REPLAY** | Rejeu de Signature Inter-Chaînes (Missing ChainId) | Challenge ou signature EIP-191/712 sans vérification du chain_id ou de contrat vérificateur. |
| 🟠 **HIGH** | `CVE-WEB3-NONCE-RACE` | **CWE-362 / TOCTOU-NONCE** | Race Condition sur Nonce Web3 | Consommation ou suppression du nonce après l'action plutôt qu'avant (Time-Of-Check to Time-Of-Use). |
| 🔴 **CRITICAL** | `CVE-WEB3-AUTH-BYPASS-MOCK` | **CWE-287 / CRYPTO-SIGNATURE-BYPASS** | Contournement Cryptographique & Signature Mock / Zero-Address | Acceptation de signature factice (0xmock, bypass_sig) ou ecrecover sans rejet explicite de l'adresse nulle (0x0000...0000), permettant l'usu... |
| 🔴 **CRITICAL** | `CVE-WEB3-ORACLE-SPOT-MANIPULATION` | **CWE-682 / FLASH-LOAN-ORACLE-MANIPULATION** | Manipulation d'Oracle Spot & Flash Loan AMM | Lecture directe du cours sur les réserves spot d'une pool AMM (getReserves, slot0, balanceOf pair) sans oracle TWAP ni flux Chainlink avec c... |
| 🔴 **CRITICAL** | `CVE-WEB3-TX-ORIGIN-PHISHING` | **CWE-302 / TX-ORIGIN-AUTHORIZATION-BYPASS** | Contournement d'Accès par Hameçonnage (tx.origin vs msg.sender) | Authentification administrative ou autorisation de transfert via tx.origin au lieu de msg.sender, permettant à un contrat intermédiaire tier... |
| 🔴 **CRITICAL** | `CVE-WEB3-UNCHECKED-TRANSFER-RETURN` | **CWE-252 / SILENT-TRANSFER-FAILURE-USDT** | Échec Silencieux des Tokens Non-Standard (USDT/BNB Missing Return) | Appel de transfer() ou transferFrom() sans SafeERC20 : USDT et tokens non conformes ne renvoient pas de booléen, provoquant un revert systém... |
| 🟠 **HIGH** | `CVE-WEB3-PERMIT-FRONT-RUNNING` | **CWE-362 / MEMPOOL-SIGNATURE-GRIEFING** | Vol & Griefing de Signature ERC-2612 Permit dans le Mempool | Exécution de token.permit() sans try/catch dans une fonction de dépôt : un bot mempool peut front-runner la signature et provoquer un revert... |
| 🔴 **CRITICAL** | `CVE-WEB3-HOOK-REENTRANCY` | **CWE-841 / ERC777-ERC1363-HOOK-REENTRANCY** | Réentrance Invisible via Hooks ERC-777 / ERC-1363 (tokensReceived) | Transfert de tokens ERC-20 sans ReentrancyGuard : les tokens compatibles ERC-777 ou ERC-1363 invoquent un callback automatique sur le destin... |
| 🔴 **CRITICAL** | `CVE-VM-ACCOUNT-MISSING-OWNER` | **CWE-284 / VM-ACCOUNT-OWNER-CONFUSION** | Confusion de Compte VM (Missing Account Owner Check) | Programme VM à modèle de comptes en Rust acceptant un AccountInfo sans vérifier que account.owner == program_id, permettant à un attaquant d... |
| 🔴 **CRITICAL** | `CVE-WEB3-READONLY-REENTRANCY` | **CWE-841 / READONLY-REENTRANCY-LP-ORACLE** | Réentrance en Lecture Seule sur Oracle LP (Curve / Balancer Virtual Price) | Interrogation du virtual_price ou de réserves d'une pool LP dans une fonction view/read-only non protégée pendant un retrait de liquidité en... |
| 🔴 **CRITICAL** | `CVE-WEB3-DIRTY-MEMORY-ASSEMBLY` | **CWE-682 / CWE-119-DIRTY-BITS-ASSEMBLY** | Corruption de Mémoire Assembly Solidity (Dirty High-Order Bits) | Manipulation en inline assembly (mload, mstore) de types de moins de 32 octets (bool, address, uint8) sans masque binaire (and(val, 0xff)), ... |
| 🔴 **CRITICAL** | `CVE-WEB3-ERC4626-INFLATION-ATTACK` | **CWE-682 / ERC4626-FIRST-DEPOSITOR-DONATION** | Attaque par Inflation de Vault ERC-4626 (Donation Front-Running) | Calcul de parts dans un Vault ERC-4626 sans parts virtuelles (_decimalsOffset) : le premier déposant mint 1 wei de part puis fait une donati... |
| 🔴 **CRITICAL** | `CVE-WEB3-VAULT-ROUNDING-DIRECTION` | **CWE-682 / INCORRECT-CONVERSION-ROUNDING** | Direction d'Arrondi Inversée ERC-4626 / Vault (Micro-Minting Exploit) | Fonction de conversion ERC-4626 (convertToShares, convertToAssets, previewDeposit, etc.) utilisant une division brute / ou Rounding.Up au li... |
| 🟠 **HIGH** | `CVE-WEB3-STALE-ORACLE-HEARTBEAT` | **CWE-682 / STALE-ORACLE-ROUND-DATA** | Oracle de Prix Sans Vérification de Fraîcheur / Heartbeat (Chainlink / Redstone) | Appel à latestRoundData() sans vérification de updatedAt == 0, block.timestamp - updatedAt > HEARTBEAT ou answeredInRound < roundId : risque... |
| 🔴 **CRITICAL** | `CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR` | **CWE-284 / UNPROTECTED-FLASHLOAN-CALLBACK** | Callback Flash Loan / Hook Sans Contrôle d'Accès msg.sender | Fonction de callback (onFlashLoan, uniswapV3SwapCallback, etc.) publique ou externe sans vérification require(msg.sender == authorizedPool) ... |
| 🟠 **HIGH** | `CVE-WEB3-SIGNATURE-DEADLINE-MISSING` | **CWE-613 / MISSING-SIGNATURE-DEADLINE** | Signature EIP-712 / Permit Sans Date d'Expiration (Mempool Holding Attack) | Fonction consommant une signature cryptographique (ecrecover, ECDSA.recover) sans vérifier require(block.timestamp <= deadline) : des valida... |
| 🔴 **CRITICAL** | `CVE-WEB3-HOOK-DEFERRED-CHECKS-DESYNC` | **CWE-841 / TRANSIENT-LIQUIDITY-INVARIANT-VIOLATION** | Désynchronisation EVC / ChecksDeferred dans HookTarget (Flash Invariant Bypass) | Hook de Vault ou contrat périphérique exécutant des transferts/appels externes ou lisant l'état de liquidité sans s'assurer que les vérifica... |
| 🟠 **HIGH** | `CVE-WEB3-DECIMAL-SCALING-TRUNCATION` | **CWE-682 / ORACLE-DECIMAL-SCALING-UNDERFLOW** | Erreur de Normalisation de Décimales d'Oracle / Collatéral (Troncature & Underflow) | Mise à l'échelle de prix ou de collatéral supposant statiquement 18 décimales (ex: 10**(18 - decimals) ou division brute / 10**decimals) vul... |
| 🔴 **CRITICAL** | `CVE-WEB3-FORK-REWARD-STAKING-DILUTION` | **CWE-682 / REWARD-HARVEST-BEFORE-UPDATE** | Dilution de Récompenses & Staking Reentrancy dans Fork DeFi (Points / Yield Farm) | Distribution ou récolte de récompenses (rewardToken.transfer, mint) lors d'un dépôt/retrait sans mise à jour préalable de l'accumulateur (ac... |
| 🔴 **CRITICAL** | `CVE-WEB3-EIP1153-TRANSIENT-COMPOSABILITY-LEAK` | **EVM-CANCUN-CRITICAL / TSTORE-UNCLEARED-COMPOSABILITY-POLLUTION** | Pollution de Mémoire Transitoire EIP-1153 Inter-Appels (Dirty TSTORE en Multicall / Batch) | Écriture dans le stockage transitoire EVM (opcode tstore ou variable transient) sans remise à zéro explicite avant la fin de l'appel : dans ... |
| 🔴 **CRITICAL** | `CVE-WEB3-EIP7702-EOA-CODE-BYPASS-COLLISION` | **EVM-PECTRA-CRITICAL / EIP7702-EXTCODESIZE-BYPASS-AND-SLOT-COLLISION** | Contournement Anti-Contrat (extcodesize == 0 / code.length == 0) & Collision de Slots EIP-7702 | Vérification d'humanité/EOA basée sur msg.sender.code.length == 0 ou extcodesize == 0, ou contrat de délégation EOA EIP-7702 sans stockage i... |
| 🔴 **CRITICAL** | `CVE-WEB3-ERC4337-PAYMASTER-GAS-SIPHON` | **AA-ERC4337-CRITICAL / PAYMASTER-POSTOP-UNBOUNDED-GAS-GRIEFING** | Siphonnage de Trésorerie Paymaster ERC-4337 par Griefing de postOp & Gaz Non Borné | Paymaster ERC-4337 validant validatePaymasterUserOp sans plafonner maxCost/postOpGasLimit ou effectuant un transferFrom ERC-20 uniquement da... |
| 🔴 **CRITICAL** | `CVE-WEB3-OMNICHAIN-LZ-CHANNEL-BRICKING` | **OMNICHAIN-CRITICAL / CROSS-CHAIN-UNCAUGHT-REVERT-FUND-BURN** | Destruction de Fonds Brûlés & Paralysie de Canal Cross-Chain (_lzReceive / _ccipReceive Sans Quarantaine) | Réception de message inter-chaînes (_lzReceive, lzReceive, _ccipReceive) exécutant un transfert ou callback externe sans isolation try/catch... |
| 🔴 **CRITICAL** | `CVE-WEB3-EIP150-63-64-GAS-STIPEND-GRIEFING` | **EVM-INTERNALS-CRITICAL / EIP150-63-64TH-SUBCALL-OOG-SILENT-BYPASS** | Sabotage Silencieux de Sous-Appel par la Règle des 63/64 de Gaz EIP-150 (Relayer / MetaTx OOG Griefing) | Exécution d'un sous-appel externe (.call ou try/catch) suivie de la validation d'un nonce ou du marquage d'un message comme exécuté sans vér... |
| 🔴 **CRITICAL** | `CVE-WEB3-ZK-SNARK-FIELD-OVERFLOW-ALIASING` | **ZK-CRYPTO-CRITICAL / BN254-SCALAR-FIELD-MODULO-ALIASING** | Double-Spend de Preuve ZK-SNARK par Aliasing Modulo le Corps Scalaire BN254 (5x Nullifier Overflow) | Vérificateur ZK (Groth16 / Plonk / Circom) acceptant des entrées publiques ou nullifiers uint256 sans vérifier input[i] < SNARK_SCALAR_FIELD... |
| 🔴 **CRITICAL** | `CVE-WEB3-MERKLE-SECOND-PREIMAGE-LEAF-NODE-COLLISION` | **CRYPTO-CRITICAL / MERKLE-64BYTE-LEAF-INTERNAL-NODE-SPOOF** | Falsification d'Arbre de Merkle par Seconde Pré-Image sur Feuille de 64 Octets (Leaf-Node Collision) | Hachage d'une feuille de Merkle via un simple keccak256(abi.encode(addr, amount)) de 64 octets sans double hachage bytes.concat(keccak256(..... |
| 🔴 **CRITICAL** | `CVE-WEB3-CREATE2-METAMORPHIC-REINIT-RUG` | **EVM-BYTECODE-CRITICAL / CREATE2-UNSALTED-SENDER-FRONT-RUN-AND-METAMORPHIC** | Détournement de Déploiement Déterministe CREATE2 (Salt Sans msg.sender & Bytecode Métamorphique) | Déploiement CREATE2 utilisant un salt fourni par l'utilisateur sans dérivation keccak256(abi.encode(msg.sender, salt)) ou chargeant son init... |

---

## 🛡️ PHP / Web Security (25 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-PHP-SQL-INJECTION` | **CWE-89 / SQL-INJECTION** | Injection SQL Directe via Concaténation de Variables Externes | Concaténation directe de données d'entrée ($_GET, $_POST, $_REQUEST, $_COOKIE, variables non assainies) dans une requête SQL (PDO::query, PD... |
| 🔴 **CRITICAL** | `CVE-PHP-COMMAND-INJECTION` | **CWE-78 / OS-COMMAND-INJECTION** | Injection de Commande Système OS Arbitraire (RCE) | Exécution de commandes système OS via system(), exec(), passthru(), shell_exec(), proc_open(), popen() ou backticks recevant des variables e... |
| 🔴 **CRITICAL** | `CVE-PHP-EVAL-CODE-INJECTION` | **CWE-94 / EVAL-CODE-INJECTION** | Exécution de Code PHP Dynamique Arbitraire (eval / create_function) | Évaluation dynamique de chaînes comme code PHP via eval(), create_function(), assert() dynamique ou preg_replace() avec modificateur /e : pr... |
| 🔴 **CRITICAL** | `CVE-PHP-UNSERIALIZE-RCE` | **CWE-502 / OBJECT-INJECTION-RCE** | Désérialisation Non Sécurisée d'Objets PHP (POP Gadget Chain RCE) | Désérialisation d'un flux externe via unserialize() sans restriction ['allowed_classes' => false] : instanciation malveillante de classes et... |
| 🔴 **CRITICAL** | `CVE-PHP-FILE-INCLUSION-RFI-LFI` | **CWE-98 / DYNAMIC-FILE-INCLUSION** | Inclusion Dynamique de Fichier Local ou Distant (LFI / RFI) | Inclusion dynamique de script via include, include_once, require, require_once alimentée par des entrées utilisateur ($_GET, $_POST, variabl... |
| 🟠 **HIGH** | `CVE-PHP-PATH-TRAVERSAL` | **CWE-22 / ARBITRARY-FILE-ACCESS** | Traversée de Répertoire & Manipulation Arbitraire de Fichier (Path Traversal) | Accès, lecture ou suppression de fichiers système hors du répertoire autorisé via file_get_contents(), readfile(), file_put_contents(), unli... |
| 🟠 **HIGH** | `CVE-PHP-XSS-UNESCAPED-OUTPUT` | **CWE-79 / REFLECTED-STORED-XSS** | Affichage Non Échappé de Données Utilisateur (Cross-Site Scripting XSS) | Affichage direct dans le document HTTP (echo, print, <?=) de superglobales ($_GET, $_POST, $_REQUEST, $_SERVER['PHP_SELF']) sans échappement... |
| 🟠 **HIGH** | `CVE-PHP-SSRF-CURL-STREAM` | **CWE-918 / SERVER-SIDE-REQUEST-FORGERY** | Requête Réseau Sortante Vers Hôte Non Filtré (SSRF) | Émission de requêtes HTTP sortantes via curl_exec(), file_get_contents($url) ou fopen($url) vers une URL fournie par le client sans filtrage... |
| 🔴 **CRITICAL** | `CVE-PHP-UNRESTRICTED-FILE-UPLOAD` | **CWE-434 / UNRESTRICTED-FILE-UPLOAD** | Téléversement de Fichier Sans Restriction d'Extension (Webshell Upload) | Déplacement de fichier téléversé via move_uploaded_file() en conservant directement le nom d'origine client ($_FILES['...']['name']) sans vé... |
| 🟠 **HIGH** | `CVE-PHP-XXE-LIBXML-INJECTION` | **CWE-611 / XML-EXTERNAL-ENTITY-INJECTION** | Parsing XML Sans Désactivation des Entités Externes (XXE) | Analyse de flux XML via simplexml_load_string(), simplexml_load_file() ou DOMDocument::loadXML() avec entités externes activées (LIBXML_NOEN... |
| 🟠 **HIGH** | `CVE-PHP-SESSION-FIXATION-NO-REGEN` | **CWE-384 / SESSION-FIXATION** | Affectation d'Authentification en Session Sans Régénération d'ID | Élévation de privilèges ou enregistrement d'état authentifié dans $_SESSION['user_id'] ou $_SESSION['auth'] sans appel préalable à session_r... |
| 🟠 **HIGH** | `CVE-PHP-COOKIE-SENSITIVE-PLAINTEXT` | **CWE-312 / INSECURE-COOKIE-TRANSMISSION** | Émission de Cookie Sensible Sans Flags Secure / HttpOnly | Création de cookies d'authentification ou tokens de session via setcookie() ou session_set_cookie_params() avec drapeaux secure ou httponly ... |
| 🟠 **HIGH** | `CVE-PHP-FILE-LOCKING-TOCTOU` | **CWE-362 / TOCTOU-FILE-RACE-CONDITION** | Condition de Course TOCTOU sur Fichier Sensible Sans Verrou Atomique | Vérification préalable d'un fichier (file_exists / is_file) immédiatement suivie d'une écriture sans verrou exclusif flock($fp, LOCK_EX) ni ... |
| 🟠 **HIGH** | `CVE-PHP-BROKEN-CRYPTO-MD5-SHA1` | **CWE-327 / BROKEN-CRYPTOGRAPHIC-HASH** | Hachage Cryptographique Obsolète pour Mots de Passe ou Secrets (MD5 / SHA1) | Utilisation d'algorithmes de hachage rapides obsolètes (md5(), sha1(), crypt()) pour le traitement de mots de passe, clés secrètes ou jetons... |
| 🟠 **HIGH** | `CVE-PHP-HEADER-INJECTION-CRLF` | **CWE-113 / HTTP-RESPONSE-SPLITTING** | Injection d'En-têtes HTTP (CRLF Injection & Open Redirect) | Émission d'en-têtes HTTP via header() incluant des données utilisateur ($_GET, $_POST, $target) sans assainissement des sauts de ligne (\\r,... |
| 🔴 **CRITICAL** | `CVE-PHP-VARIABLE-OVERWRITE-EXTRACT` | **CWE-621 / VARIABLE-OVERWRITE-EXTRACT** | Écrasement Arbitraire de Variables via extract() ou Variables Dynamiques | Importation non sécurisée de variables externes via extract($_GET), extract($_POST), extract($_REQUEST) sans l'indicateur de sécurité EXTR_S... |
| 🔴 **CRITICAL** | `CVE-PHP-MAIL-PARAM-INJECTION` | **CWE-88 / MAIL-PARAMETER-INJECTION** | Injection de Paramètres Sendmail via mail() (5ème Argument RCE) | Utilisation de la fonction native mail() avec un 5ème paramètre ($additional_params) dynamique recevant des variables externes non protégées... |
| 🔴 **CRITICAL** | `CVE-PHP-TEMPLATE-INJECTION-SSTI` | **CWE-1336 / SERVER-SIDE-TEMPLATE-INJECTION** | Injection de Modèle Côté Serveur (SSTI dans Twig / Blade / Smarty) | Compilation ou rendu dynamique de modèles de templates (Twig::createTemplate, Blade::render, Smarty::display('string:...')) à partir de chaî... |
| 🟠 **HIGH** | `CVE-PHP-DISABLED-SSL-VERIFY` | **CWE-295 / DISABLED-SSL-VERIFICATION** | Désactivation de la Vérification des Certificats SSL/TLS | Désactivation explicite du contrôle de validité des certificats TLS/SSL via CURLOPT_SSL_VERIFYPEER à false, CURLOPT_SSL_VERIFYHOST à 0 ou co... |
| 🟠 **HIGH** | `CVE-PHP-TYPE-JUGGLING-AUTH` | **CWE-697 / LOOSE-COMPARISON-TYPE-JUGGLING** | Comparaison Faible (Type Juggling) sur Secret, Token ou Mot de Passe | Utilisation de comparaisons non strictes (==, !=, strcmp(...) == 0) lors de la validation d'un mot de passe, jeton HMAC, secret ou signature... |
| 🟠 **HIGH** | `CVE-PHP-WEAK-PRNG-SECRETS` | **CWE-338 / CRYPTOGRAPHICALLY-WEAK-PRNG** | Générateur Aléatoire Non Cryptographique pour Secrets ou Jetons | Génération d'identifiants de sécurité (tokens CSRF, jetons de réinitialisation de mot de passe, clés d'API, sessions) à l'aide de générateur... |
| 🟠 **HIGH** | `CVE-PHP-LDAP-INJECTION` | **CWE-90 / LDAP-INJECTION** | Injection LDAP Directe via Concaténation de Filtres | Construction dynamique de filtres d'annuaire LDAP via ldap_search(), ldap_list() ou ldap_read() avec concaténation directe d'entrées utilisa... |
| 🟠 **HIGH** | `CVE-PHP-NOSQL-INJECTION` | **CWE-943 / NOSQL-INJECTION-MONGODB** | Injection d'Opérateurs NoSQL MongoDB via Tableaux Non Castés | Passage direct de données d'entrée ($_POST, $_GET, $_REQUEST) sous forme de critères de recherche dans des requêtes MongoDB ($collection->fi... |
| 🟠 **HIGH** | `CVE-PHP-ZIP-SLIP-EXTRACTION` | **CWE-770 / ZIP-SLIP-ARBITRARY-EXTRACTION** | Décompression d'Archive Non Sécurisée (Vulnérabilité Zip Slip) | Extraction d'archives compressées via ZipArchive::extractTo() sans vérification de confinement des chemins relatifs (séquences ../) : un fic... |
| 🟠 **HIGH** | `CVE-PHP-EXPOSED-DEBUG-CONFIG` | **CWE-209 / SENSITIVE-INFO-DEBUG-EXPOSURE** | Exposition de Configuration de Débogage & Messages d'Erreur (phpinfo / display_errors) | Activation explicite de l'affichage des erreurs à l'utilisateur (ini_set('display_errors', '1')) ou appel à la fonction de diagnostic phpinf... |

---

## 🛡️ SAST / Injection (17 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `SAST_RAW_SQL_INJECTION` | **CWE-89 / SQL-INJECTION** | Injection SQL Brute Dynamique | Concaténation dynamique de requêtes SQL dans les couches protégées au lieu de requêtes préparées. |
| 🔴 **CRITICAL** | `SAST_HARDCODED_SECRET` | **CWE-798 / HARDCODED-SECRET** | Identifiant ou Clé Secrète en Clair | Clé privée, token API ou secret cryptographique hardcodé en clair dans le code source. |
| 🔴 **CRITICAL** | `CVE-DB-REDIS-UNAUTH-KEYS-DOS` | **CWE-306 / CWE-400-REDIS-KEYS-DOS** | DoS Redis O(N) (Commande KEYS *) & Socket Cache Non Authentifié | Exécution de la commande bloquante O(N) KEYS sur Redis en production (au lieu de SCAN) ou exposition d'un serveur RESP/TCP sans authentifica... |
| 🔴 **CRITICAL** | `CVE-RUST-UNSAFE-MEM` | **CWE-119 / CVE-RUST-UNSAFE-MEM** | Bloc Unsafe & Pointeurs Bruts Rust | Usage de unsafe, pointeurs bruts *mut T ou std::mem::transmute contournant le borrow-checker. |
| 🟠 **HIGH** | `CVE-RUST-PANIC-DOS` | **CWE-248 / CVE-RUST-PANIC-DOS** | Panique / Crash Process sur Entrée Invalide | .unwrap() ou .expect() directs sur des entrées réseau/utilisateurs dans la couche serveur. |
| 🔴 **CRITICAL** | `CVE-RUST-CMD-INJ` | **CWE-78 / CVE-RUST-CMD-INJ** | Injection de Commande Système OS (Rust) | Exécution de processus via Command::new avec arguments non filtrés. |
| 🔴 **CRITICAL** | `CVE-PY-DESERIALIZE-RCE` | **CWE-502 / CVE-PY-DESERIALIZE-RCE** | Désérialisation Python Non Sécurisée (Pickle/YAML/Eval) | Exécution de code arbitraire via pickle.loads(), yaml.load() sans SafeLoader ou eval(). |
| 🔴 **CRITICAL** | `CVE-PY-SHELL-INJ` | **CWE-78 / CVE-PY-SHELL-INJ** | Injection Shell Python (subprocess shell=True / os.system) | Exécution de commande shell en Python avec shell=True ou os.system(). |
| 🟠 **HIGH** | `CVE-PY-SSRF-UNVALIDATED` | **CWE-918 / CVE-PY-SSRF-UNVALIDATED** | Requête HTTP Sortante Non Filtrée (SSRF Python) | Requête requests/httpx vers une URL dynamique sans liste blanche ni blocage d'IP privées. |
| 🟠 **HIGH** | `CVE-PY-UNTYPED-PAYLOAD` | **CWE-20 / CVE-PY-UNTYPED-PAYLOAD** | Payload API Python Non Typé (dict brut sans Pydantic) | Route API Python acceptant un dict brut au lieu d'un schéma Pydantic v2 strict. |
| 🔴 **CRITICAL** | `CVE-PY-PATH-TRAVERSAL-ARCHIVE` | **CWE-22 / PATH-TRAVERSAL-ZIPSLIP** | Path Traversal & Extraction d'Archive Non Bornée (ZipSlip Python) | Ouverture de fichier dynamique (open, send_file) ou extraction d'archive (extractall, unpack_archive) sans vérification Path.resolve().is_re... |
| 🔴 **CRITICAL** | `CVE-PY-JINJA2-SSTI-INJECTION` | **CWE-1336 / JINJA2-SSTI-RCE** | Injection de Template Côté Serveur (Jinja2 SSTI RCE) | Concaténation de variables ou f-strings non échappées dans render_template_string() ou jinja2.Template(), permettant l'exécution de code arb... |
| 🔴 **CRITICAL** | `CVE-PY-YAML-UNSAFE-LOAD` | **CWE-502 / YAML-UNSAFE-DESERIALIZATION** | Désérialisation Non Sécurisée PyYAML (yaml.load sans SafeLoader) | Appel à yaml.load() sans Loader=SafeLoader ou yaml.unsafe_load(), permettant à un document YAML forgé d'instancier des objets Python arbitra... |
| 🔴 **CRITICAL** | `CVE-PY-SQL-ALCHEMY-TEXT-INJECTION` | **CWE-89 / SQLALCHEMY-RAW-TEXT-INJECTION** | Injection SQL Brute dans SQLAlchemy text() (Bypass ORM) | Utilisation de sqlalchemy.text() avec concaténation dynamique (f-string, +, %) au lieu de paramètres liés (:param / bindparams), réintroduis... |
| 🔴 **CRITICAL** | `CVE-PY-JWT-NONE-ALGORITHM` | **CWE-327 / JWT-UNVERIFIED-NONE-ALGORITHM** | Contournement d'Authentification JWT (Algorithme 'none' ou Signature Non Vérifiée) | Décodage de jeton JWT via jwt.decode() avec verify=False, verify_signature=False ou tolérance de l'algorithme 'none', permettant de forger n... |
| 🟠 **HIGH** | `CVE-PY-PARAMIKO-MISSING-HOST-KEY` | **CWE-295 / PARAMIKO-AUTOADD-HOSTKEY-MITM** | Connexion SSH Paramiko Vulnérable aux Attaques MITM (AutoAddPolicy) | Configuration de Paramiko avec AutoAddPolicy(), acceptant aveuglément n'importe quelle clé d'hôte SSH inconnue et exposant le trafic à une i... |
| 🟠 **HIGH** | `CVE-PY-XML-XXE-ENTITY-EXPANSION` | **CWE-611 / XML-EXTERNAL-ENTITY-XXE** | Injection d'Entités Externes XML (XXE / Entity Expansion) | Parsing XML avec ElementTree ou minidom sans désactiver la résolution des entités externes ou sans defusedxml, permettant la lecture de fich... |

---

## 🛡️ Rust / Network (14 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-DESER-BOMB-OOM` | **CWE-789 / LENGTH-PREFIX-OOM-BOMB** | Bombe d'Allocation Réseau (Length-Prefix OOM Abort) | Désérialisation binaire (bincode, rmp_serde) ou Vec::with_capacity(wire_len) sans .with_limit() : un préfixe forgé de 8 octets provoque un a... |
| 🟠 **HIGH** | `CVE-RUST-CANCEL-SAFETY-LOSS` | **CWE-662 / TOKIO-SELECT-CANCEL-LOSS** | Perte Silencieuse de Trame TCP (tokio::select! Cancel-Safety) | Utilisation de futures non cancel-safe (read_exact, write_all, read_to_end) dans tokio::select!, corrompant le flux TCP si une autre branche... |
| 🟠 **HIGH** | `CVE-RUST-TIMING-SIDECHANNEL` | **CWE-208 / TIMING-SIDE-CHANNEL** | Fuite Temporelle Réseau (Non-Constant-Time Secret Compare) | Comparaison de secrets, nonces, HMAC ou signatures via == ou != (court-circuité par memcmp SIMD) au lieu de subtle::ConstantTimeEq::ct_eq(). |
| 🟠 **HIGH** | `CVE-NET-SLOWLORIS-KEEPALIVE-DOS` | **CWE-400 / SLOWLORIS-SOCKET-EXHAUSTION** | Épuisement de Sockets TCP/WS (Slowloris & Missing Timeout) | Acceptation de socket (accept_async, TcpListener::accept, axum::serve) sans tokio::time::timeout ni TimeoutLayer sur le handshake réseau. |
| 🔴 **CRITICAL** | `CVE-NET-SSRF-DNS-REBIND` | **CWE-918 / SSRF-DNS-REBINDING** | SSRF & DNS Rebinding TOCTOU sur Socket Sortant | Connexion sortante (reqwest, TcpStream::connect) vers une cible dynamique sans filtrage DNS post-résolution (is_loopback, is_private, 169.25... |
| 🔴 **CRITICAL** | `CVE-RUST-BLOCKING-IN-ASYNC` | **CWE-400 / ASYNC-EXECUTOR-STARVATION** | Famine du Runtime Async (I/O ou Sleep Bloquant dans async fn) | Appel bloquant synchrone (std::fs, std::thread::sleep, std::net::TcpStream) directement dans un contexte async fn sans tokio::task::spawn_bl... |
| 🟠 **HIGH** | `CVE-RUST-INTEGER-OVERFLOW-CAST` | **CWE-190 / AS-CAST-INTEGER-TRUNCATION** | Troncature Silencieuse par Cast de Conversion Entière (as u32 / as u16 / as u8) | Conversion de taille de buffer, solde ou entier 64-bit avec l'opérateur 'as' sans contrôle de bornes, provoquant une troncature silencieuse ... |
| 🔴 **CRITICAL** | `CVE-RUST-TRANSMUTE-SIZE-MISMATCH` | **CWE-843 / UNSAFE-TRANSMUTE-TYPE-CONFUSION** | Transmutation Arbitraire de Types Mémoire (std::mem::transmute) | Appel à std::mem::transmute() sans vérification de taille ou layout mémoire équivalent, entraînant un comportement indéfini (UB) et une corr... |
| 🟠 **HIGH** | `CVE-RUST-FORGOTTEN-MEM-LEAK` | **CWE-775 / MEM-FORGET-RESOURCE-LEAK-DEADLOCK** | Contournement RAII & Fuite Critique par std::mem::forget (Deadlock de Verrou) | Appel explicite à std::mem::forget() sur un garde de synchronisation (MutexGuard, RwLock) ou socket, empêchant le Drop de s'exécuter et caus... |
| 🟠 **HIGH** | `CVE-RUST-UNCHECKED-INDEX-SLICING` | **CWE-125 / UNCHECKED-SLICE-INDEX-PANIC-DOS** | Indexation Directe de Tranche Réseau Sans Contrôle (Panic DOS sur Paquet Forgé) | Indexation directe par tranche [offset..offset + len] sur un buffer de trame réseau brut sans vérification .get(..) préalable, provoquant un... |
| 🟠 **HIGH** | `CVE-RUST-THREAD-POISON-IGNORED` | **CWE-391 / SWALLOWED-MUTEX-POISON-INVARIANT** | Verrou Mutex Empoisonné Ignoré Silencieusement (Swallowed PoisonError) | Récupération aveugle d'un Mutex empoisonné via .unwrap_or_else(|e| e.into_inner()), continuant d'opérer sur des données partagées compromise... |
| 🔴 **CRITICAL** | `CVE-RUST-FFI-UNCHECKED-NULL-PTR` | **CWE-476 / FFI-UNCHECKED-NULL-PTR-DEREF** | Déréférencement FFI de Pointeur Brut Sans Vérification de Nullité | Déréférencement direct de pointeur brut (*const T / *mut T) dans un bloc unsafe sans assertion préalable ptr.is_null() ou sans NonNull, caus... |
| 🔴 **CRITICAL** | `CVE-RUST-PIN-STRUCTURAL-UNPIN-SOUNDNESS-UB` | **RUST-INTERNALS-CRITICAL / PIN-GET-UNCHECKED-MUT-SOUNDNESS-BREAK** | Rupture d'Invariant Pinning Async (Pin::get_unchecked_mut / Self-Referential Move UB) | Projection manuelle Pin::get_unchecked_mut ou Pin::new_unchecked sans pin_project! sur une Future ou structure auto-référentielle : un mem::... |
| 🔴 **CRITICAL** | `CVE-L1-BORSH-UNBOUNDED-DESER-OOM` | **L1-P2P-CONSENSUS-CRITICAL / BORSH-ZSTD-WIRE-ALLOCATION-BOMB** | Bombe de Désérialisation P2P Borsh / Zstd sur Paquet ou StateWitness Non Authentifié | Désérialisation Borsh (BorshDeserialize::try_from_slice) ou décompression zstd::decode_all sur une trame P2P ou un ChunkStateWitness sans pl... |

---

## 🛡️ Front-End / DOM (13 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-JS-EVENTLOOP-REDOS-JSON` | **CWE-1333 / CWE-770-EVENTLOOP-DOS** | Blocage de l'Event-Loop Node.js (Stream HTTP Non Borné & ReDoS) | Accumulation de flux HTTP req.on('data') sans plafond d'octets (MAX_BODY_SIZE) ou RegExp dynamique non bornée, permettant de geler l'Event-L... |
| 🟠 **HIGH** | `CVE-JS-XSS-DOM` | **CWE-79 / CVE-JS-XSS-DOM** | Injection DOM / XSS dans l'UI | Assignation brute via innerHTML, dangerouslySetInnerHTML ou document.write(). |
| 🟠 **HIGH** | `CVE-JS-PROTO-POLLUTION` | **CWE-1321 / CVE-JS-PROTO-POLLUTION** | Pollution de Prototype / Payload Non Typé | Ingestion de payload : any ou req.body sans validation de schéma strict (Zod/contracts). |
| 🔴 **CRITICAL** | `CVE-JS-CMD-INJ` | **CWE-78 / CVE-JS-CMD-INJ** | Injection de Commande Shell (Node.js) | child_process.exec ou execSync exécutant des chaînes de commande dynamiques. |
| 🔴 **CRITICAL** | `CVE-JS-CODE-INJ` | **CWE-94 / CVE-JS-CODE-INJ** | Évaluation Dynamique de Code (eval / new Function) | Utilisation de eval() ou new Function() permettant l'exécution de code JavaScript arbitraire. |
| 🟡 **MEDIUM** | `CVE-CLIENT-UNSAFE-ASSETS` | **CWE-829 / UNSAFE-ASSET-LOADING** | Chargement d'Asset Graphique Non Validé | Chargement dynamique de texture PixiJS / Three.js via Assets.load() sans domaine de confiance ou validation CDN. |
| 🔴 **CRITICAL** | `CVE-JS-VM-SANDBOX-ESCAPE` | **CWE-94 / VM-SANDBOX-ESCAPE-RCE** | Échappement de Bac à Sable Node.js vm (Évasion de Sandbox & RCE) | Exécution de code via le module natif vm (vm.runInContext, vm.runInNewContext, vm.Script). Le module vm n'est pas un mécanisme de sécurité e... |
| 🔴 **CRITICAL** | `CVE-JS-SSRF-DYNAMIC-AXIOS-FETCH` | **CWE-918 / AXIOS-FETCH-OUTBOUND-SSRF** | SSRF Sortant via Requête HTTP Dynamique (axios / fetch / got) | Envoi de requête HTTP sortante via axios, fetch, got ou http.get avec une URL dynamique non validée, permettant d'atteindre les métadonnées ... |
| 🔴 **CRITICAL** | `CVE-JS-PATH-TRAVERSAL-SENDFILE` | **CWE-22 / SENDFILE-PATH-TRAVERSAL** | Lecture Arbitraire de Fichier & Path Traversal (res.sendFile / fs.readFile) | Transmission de fichier dynamique avec res.sendFile() ou fs.readFile() basé sur des entrées utilisateurs (req.params/req.query) sans confine... |
| 🟠 **HIGH** | `CVE-JS-REGEX-DOS-DYNAMIC-INPUT` | **CWE-1333 / DYNAMIC-REGEXP-CATASTROPHIC-REDOS** | Déni de Service ReDoS par Expression Régulière Dynamique | Instanciation de new RegExp() directement depuis une variable utilisateur ou chaîne non échappée, permettant d'injecter des motifs à rebrous... |
| 🟠 **HIGH** | `CVE-JS-CORS-WILDCARD-CREDENTIALS` | **CWE-942 / CORS-REFLECTED-ORIGIN-WITH-CREDENTIALS** | Configuration CORS Dangereuse avec Origine Réfléchie & Identifiants | Configuration CORS avec credentials: true associée à origin: true ou reflétant l'en-tête Origin client, permettant à un site web tiers malve... |
| 🟠 **HIGH** | `CVE-JS-UNHANDLED-PROMISE-CRASH` | **CWE-755 / ASYNC-ROUTE-UNHANDLED-REJECTION** | Rejet de Promesse Non Intercepté dans un Handler de Route Async (Crash Process) | Handler de route Express/Koa async sans bloc try/catch et sans transfert vers next(err), déclenchant des UnhandledPromiseRejection qui provo... |
| 🔴 **CRITICAL** | `CVE-JS-V8-TYPEDARRAY-SHARED-AB-RACE-OOB` | **V8-INTERNALS-CRITICAL / SHAREDARRAYBUFFER-TOCTOU-AND-ALLOCUNSAFE-BLEED** | Fuite de Heap V8 (Buffer.allocUnsafe) & Race Condition TOCTOU sur SharedArrayBuffer | Allocation mémoire brute Buffer.allocUnsafe() sans .fill(0) exfiltrant les clés privées de la heap V8 sur le réseau (Nodebleed), ou lecture ... |

---

## 🛡️ C / C++ / Network (10 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-CPP-NET-INTEGER-OVERFLOW-ALLOC` | **CWE-190 / INTEGER-OVERFLOW-ALLOC-OOM** | Dépassement d'Entier sur Calcul de Longueur Réseau & Allocation Tronquée | Calcul arithmétique sur la taille d'un paquet réseau (+ sizeof(Header), * count) directement dans new char[] ou malloc() sans validation de ... |
| 🟠 **HIGH** | `CVE-CPP-NET-TYPE-PUNNING-STRICT-ALIASING` | **CWE-843 / STRICT-ALIASING-TYPE-PUNNING** | Type Punning Réseau par Cast de Pointeur Brut (Strict Aliasing UB) | Cast direct d'un tampon d'octets brut (char*/uint8_t*) vers une structure de protocole (reinterpret_cast<Header*> ou (PacketHeader*)buf) : v... |
| 🔴 **CRITICAL** | `CVE-CPP-NET-ASYNC-CALLBACK-UAF` | **CWE-416 / ASYNC-EVENT-CALLBACK-UAF** | Use-After-Free par Capture de this dans Callback Réseau Asynchrone | Capture du pointeur brut [this] ou [&] dans un callback asynchrone (async_read, async_write, async_receive, uv_queue_work) sans shared_from_... |
| 🟠 **HIGH** | `CVE-CPP-NET-PARTIAL-READ-SHORT-FRAME` | **CWE-252 / UNCHECKED-SHORT-STREAM-READ** | Désynchronisation TCP par Lecture Partielle Non Bouclée (Unchecked Short Read) | Appel unique à recv(), read() ou SSL_read() supposant la réception atomique de la trame complète sans boucle de cadrage ni validation strict... |
| 🟠 **HIGH** | `CVE-CPP-NET-SOCKET-FD-LEAK-EXCEPTION` | **CWE-775 / SOCKET-FD-LEAK-EXCEPTION** | Fuite de Descripteur Socket Réseau par Absence de RAII (FD Exhaustion DoS) | Création d'un descripteur brut (socket(), accept(), open()) stocké dans un int sans garde RAII dans une fonction contenant des allocations d... |
| 🟠 **HIGH** | `CVE-CPP-NET-HOST-BYTE-ORDER-CONFUSION` | **CWE-1385 / MISSING-ENDIANNESS-CONVERSION** | Confusion d'Endianness sur Champ Longueur Réseau (Missing ntohs/ntohl) | Lecture directe d'un champ de longueur réseau (header.length, pkt->size) dans des opérations de comparaison de bornes ou d'allocation sans c... |
| 🔴 **CRITICAL** | `CVE-CPP-NET-STRING-NULL-TERMINATION-OFF-BY-ONE` | **CWE-170 / MISSING-NULL-TERMINATION-LEAK** | Absence de Null-Termination sur Chaîne Réseau Reçue (Memory Bleed) | Lecture réseau dans un buffer char[] sans forcer le byte nul terminal (buf[n] = '\\0'), suivi d'un traitement par strlen(), std::string(buf)... |
| 🔴 **CRITICAL** | `CVE-CPP-NET-FORMAT-STRING-LOGGING` | **CWE-134 / UNCONTROLLED-FORMAT-STRING-NET** | Injection de Chaîne de Formatage sur Données Réseau (Format String RCE) | Passage direct d'un buffer ou message reçu du réseau comme chaîne de format à syslog(), printf(), sprintf() ou fprintf() sans format spécifi... |
| 🟠 **HIGH** | `CVE-CPP-NET-SOCKET-RACE-DOUBLE-CLOSE` | **CWE-415 / SOCKET-FD-RACE-DOUBLE-CLOSE** | Double-Close & Race Condition sur Socket Multi-Thread (FD Re-use Hijack) | Fermeture de socket close(fd) ou closesocket(fd) dans un destructeur ou gestionnaire d'événement sans réinitialisation atomique (exchange(-1... |
| 🟠 **HIGH** | `CVE-CPP-NET-UNBOUNDED-RECURSION-TLV-PARSER` | **CWE-674 / UNBOUNDED-RECURSIVE-NET-PARSER** | Récursion Non Bornée sur Parseur de Protocole Réseau (Stack Exhaustion Crash) | Fonction de parsing récursif de paquets réseau imbriqués (TLV, JSON, BSON, expressions) s'appelant récursivement sans compteur ni plafond de... |

---

## 🛡️ MMO / Concurrency (5 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-ASYNC-MUTEX` | **CWE-667 / ASYNC-DEADLOCK** | Deadlock Asynchrone (std::sync::Mutex across .await) | Garde de Mutex synchrone conservé à travers un point .await, bloquant le thread pool Tokio. |
| 🟠 **HIGH** | `CVE-RUST-UNBOUNDED-CHANNEL` | **CWE-400 / UNBOUNDED-MEMORY-DOS** | Saturation Mémoire Canal Réseau Non Borné | Utilisation de unbounded_channel() sur les flux WebSocket ou paquets réseau risquant le crash OOM. |
| 🟠 **HIGH** | `CVE-MMO-UNCHECKED-SPEEDHACK` | **CWE-20 / UNCHECKED-SPEEDHACK** | Téléportation & Speedhack Client Non Validé | Coordonnées de joueur assignées directement depuis un paquet client sans contrôle de vitesse max (delta_t). |
| 🔴 **CRITICAL** | `CVE-MATH-UNCHECKED-BALANCE` | **CWE-190 / INTEGER-UNDERFLOW** | Underflow Arithmétique de Solde / Monnaie | Décrémentation de solde ou pièces sans saturating_sub ou checked_sub, provoquant un wrapping. |
| 🔴 **CRITICAL** | `CVE-MMO-IDOR-TRUSTED-USERID` | **CWE-639 / IDOR-CLIENT-USERID-SPOOF** | IDOR & Usurpation d'Identité Joueur via Payload Client (userId non authentifié) | Extraction directe de userId / user_id depuis le corps de requête client sans résolution ni validation du token de session côté serveur (per... |

---

## 🛡️ DeFi / Oracles (4 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-DEFI-READ-ONLY-REENTRANCY` | **CWE-841 / DEFI-READ-ONLY-REENTRANCY** | DeFi: Read-Only Reentrancy on Curve / Balancer AMM Virtual Price | Invocation of view price feeds (get_virtual_price(), getRate(), getPoolTokens()) during intermediate state without reentrancy guard or lock ... |
| 🔴 **CRITICAL** | `DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER` | **LOGIC-CRITICAL / ORACLE_MIN_MAX_CIRCUIT_BREAKER** | Absence de Contrôle des Bornes Min/Max d'Oracle (Effondrement de Prix) | Interrogation d'un flux d'oracle sans vérification que le prix retourné ne percute pas les bornes matérielles minAnswer ou maxAnswer de l'ag... |
| 🔴 **CRITICAL** | `DEFI-ORACLE-L2-SEQUENCER-RACE` | **LOGIC-CRITICAL / L2_SEQUENCER_ORACLE_RACE** | Course d'Oracle sur Séquenceur L2 Sans Période de Grâce (Arbitrum / Base / Optimism) | Lecture de prix Chainlink sur Layer 2 sans interroger le sequencerUptimeFeed et sans imposer de délai de grâce (GRACE_PERIOD_TIME) après le ... |
| 🔴 **CRITICAL** | `DEFI-TWAP-SHORT-WINDOW` | **LOGIC-CRITICAL / TWAP_SHORT_WINDOW** | Fenêtre TWAP AMM Inférieure à 30 Minutes Manipulable par MEV Multi-Blocs | Fenêtre temporelle d'oracle TWAP Uniswap v3 ou AMM inférieure à 1800 secondes (30 minutes) ou non plafonnée par un minimum strict, permettan... |

---

## 🛡️ Architecture / Boundary (4 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `BLAKE3_TAMPERED` | **CWE-353 / BLAKE3-INTEGRITY** | Altération Clandestine de Contrat BLAKE3 | Fichier README.md ou contrat de dossier modifié sans signature officielle dans aether.lock.toml. |
| 🔴 **CRITICAL** | `RULE_FORBIDDEN_IMPORT` | **CWE-1075 / ARCH-CONTRACT-VIOLATION** | Franchissement Illégal de Frontière de Couche | Import interdit violant l'étanchéité architecturale (ex: client important le domaine serveur). |
| 🟡 **MEDIUM** | `RULE_MAX_LINES` | **CWE-1075 / ARCH-BUDGET-EXCEEDED** | Dépassement de Budget de Lignes de Fichier | Le fichier dépasse la limite max_lines_per_file définie dans le contrat EARS du dossier. |
| 🟠 **HIGH** | `SAST_DIRECT_ENV_ACCESS` | **CWE-15 / DIRECT-ENV-ACCESS** | Accès Direct aux Variables d'Environnement | Accès à process.env ou std::env::var dans la couche domaine pur (100% sans I/O). |

---

## 🛡️ SAST / Injection & RCE (3 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-ARBITRARY-FILE-WRITE-TRAVERSAL` | **CWE-22 / RUST-ARBITRARY-FILE-WRITE-TRAVERSAL** | Écriture de Fichier Arbitraire / Path Traversal vers RCE (Rust) | Écriture ou création de fichier (File::create, fs::write, jemalloc prof.dump) sur un chemin dérivé d'une entrée réseau/HTTP sans validation ... |
| 🔴 **CRITICAL** | `CVE-RUST-COMMAND-INJECTION-SHELL` | **CWE-78 / RUST-COMMAND-INJECTION-SHELL** | Injection de Commande Système OS via Interpréteur Shell (sh -c / cmd /C) | Exécution de processus système via Command::new invoquant un interpréteur shell (sh, bash, cmd, powershell) avec des drapeaux d'évaluation (... |
| 🔴 **CRITICAL** | `CVE-RUST-DYNAMIC-LIB-HIJACK` | **CWE-427 / RUST-DYNAMIC-LIB-HIJACK** | Chargement Dynamique de Bibliothèque Non Sécurisé (DLL / .so Hijacking) | Chargement de module ou bibliothèque partagée dynamique (.so / .dll via libloading ou dlopen) avec un chemin relatif ou non protégé : exécut... |

---

## 🛡️ DeFi / Invariants (3 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🟠 **HIGH** | `DEFI-ASYMMETRIC-ROUNDING` | **CWE-682 / ASYMMETRIC_ROUNDING_RISK** | Arrondi Asymétrique en Faveur de l'Attaquant (Vaults / Taxe Économique) | Division entière brute ou calcul de ratio dans les conversions d'actifs/shares ou taxes économiques sans fonction d'arrondi sécurisée (mulDi... |
| 🔴 **CRITICAL** | `DEFI-STALE-CACHE-AFTER-CALL` | **CWE-362 / STALE_CACHE_TOCTOU** | Lecture d'État / Cache Périmé Après Appel Externe (VaultCache TOCTOU) | Réutilisation d'un snapshot ou cache d'état local (VaultCache, local_balance, snapshot) après un appel externe (.call, HookTarget, .await) s... |
| 🔴 **CRITICAL** | `EVC-HOOK-BYPASS` | **CWE-841 / EVC_HOOK_BYPASS_RISK** | Contournement de Collatéral / Batch Reentrancy via Hook EVC Non Sécurisé | Exécution de mutations d'état ou transferts dans un hook/callback sans vérification de l'état global du connecteur EVC ou des contrôles de l... |

---

## 🛡️ Cryptography / Zero-Knowledge (2 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-ZK-WEAK-FIAT-SHAMIR` | **CWE-347 / WEAK-FIAT-SHAMIR-TRANSCRIPT-BYPASS** | ZK-SNARKs: Weak Fiat-Shamir Transformation (Missing Public Input / Instance Binding) | Challenge sampling in non-interactive ZK proof without absorbing public inputs or instance statements into the Fiat-Shamir transcript: allow... |
| 🔴 **CRITICAL** | `CVE-ZK-UNDERCONSTRAINED-SIGNAL` | **CWE-682 / ZK-UNDERCONSTRAINED-CIRCUIT** | ZK Circuits: Underconstrained Witness Assignment without R1CS Enforcement | Witness assignment operator '<--' used in Circom or unconstrained variable in Halo2 circuit without corresponding equality constraint '===' ... |

---

## 🛡️ Account-Model VM / Smart Contracts (2 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-VM-ACCOUNT-CONFUSION-DISCRIMINATOR` | **CWE-843 / VM-ACCOUNT-CONFUSION-DISCRIMINATOR** | Account-Model VM: Missing 8-Byte Discriminator Validation (Account Cosplay) | Manual deserialization with try_from_slice() or raw account access without validating 8-byte type discriminator or account owner: allows arb... |
| 🔴 **CRITICAL** | `CVE-VM-MISSING-SIGNER-CHECK` | **CWE-287 / VM-MISSING-SIGNER-CHECK** | Account-Model VM: Privileged Authority Key Verification without is_signer Assertion | Authority or admin public key comparison (authority.key == ...) without enforcing account.is_signer check: allows unauthorized users to pass... |

---

## 🛡️ Rust / Memory Safety & RCE (2 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-UNSOUND-SLICE-FROM-RAW-PARTS` | **CWE-119 / RUST-UNSOUND-SLICE-FROM-RAW-PARTS** | Corruption Mémoire: slice::from_raw_parts sur Pointeurs Non Vérifiés | Reconstitution de structures de données ou tranches mémoire via slice::from_raw_parts ou Vec::from_raw_parts sans assertion sur la nullité (... |
| 🔴 **CRITICAL** | `CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB` | **CWE-415 / RUST-UNWIND-PREMATURE-SET-LEN-DOUBLE-FREE** | Rust Unwind Safety: Premature Vec::set_len or Unprotected ptr::read Drop-Bomb (Double-Free / Uninit Drop UB) | Appel à Vec::set_len(n) avant le remplissage effectif du tampon (read_exact, copy_nonoverlapping, boucle faillible) ou ptr::read() sans Manu... |

---

## 🛡️ Rust / Memory Safety (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🟠 **HIGH** | `CVE-RUST-UNSOUND-TRANSMUTE-ALIGNMENT` | **CWE-843 / RUST-UNSOUND-TRANSMUTE-ALIGNMENT** | Rust Soundness: Transmute or Raw Pointer Cast Violating Type Memory Alignment | Transmute or slice::from_raw_parts between types of differing alignment requirements without alignment assertion (align_of::<T>()): causes u... |

---

## 🛡️ Rust / Concurrency (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION` | **CWE-662 / RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION** | Rust Async: Cancellation-Unsafe Operation in tokio::select! Branch | Non-cancellation-safe future (write_all, read_exact, or multi-step network framing) inside tokio::select! branch: cancellation leaves stream... |

---

## 🛡️ Wasm Contracts / Storage (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🟠 **HIGH** | `CVE-WASM-UNBOUNDED-STORAGE-ITERATION` | **CWE-400 / WASM-STORAGE-BOMB** | Wasm Contracts: Unbounded KV-Storage Prefix Iteration (Gas Freeze DoS) | Iterating storage map or prefix with .range() or .prefix() without strict .take(LIMIT) pagination: state spam causes block gas limit exhaust... |

---

## 🛡️ Layer 2 / Rollups (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🟠 **HIGH** | `CVE-EVM-L2-DATA-GAS-UNDERESTIMATION` | **CWE-682 / EVM-L2-DATA-GAS-UNDERESTIMATION** | Layer 2: Unaccounted L1 Calldata / Blob Fee in Relayer Gas Calculation | L2 transaction refund or subsidization calculating cost solely via gasleft() * tx.gasprice without querying L1 GasPriceOracle: calldata-heav... |

---

## 🛡️ Web3 / Account Abstraction (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🟠 **HIGH** | `CVE-ERC4337-PAYMASTER-STORAGE-POISONING` | **CWE-667 / ERC4337-PAYMASTER-STORAGE-POISONING** | ERC-4337 Account Abstraction: Paymaster Global Storage Rule Violation (Mempool DoS) | Access or mutation of global unassociated storage in validatePaymasterUserOp: triggers ERC-4337 bundler reputation slashing and banishment o... |

---

## 🛡️ SAST / Deserialization (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🟠 **HIGH** | `CVE-RUST-UNBOUNDED-BCS-DESERIALIZE` | **CWE-502 / RUST-UNBOUNDED-BCS-DESERIALIZE** | Désérialisation Non Bornée BCS / Serde (Crash Récursif & Gadgets) | Désérialisation directe bcs::from_bytes sans contrôle de taille de buffer ni garde-fou sur la récursion : risque de Stack Overflow non rattr... |

---

## 🛡️ Move VM / Memory Safety & RCE (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-VM-NATIVE-UNSAFE-BUFFER-BOUNDS` | **CWE-787 / VM-NATIVE-UNSAFE-BUFFER-BOUNDS** | Move VM Native Extension: Manipulation Mémoire Unsafe sans Contrôle de Bornes | Dans une fonction native Move en Rust (VM framework / native), manipulation de pointeurs bruts (copy_nonoverlapping, .offset()) avec des lon... |

---

## 🛡️ Rust / Compiler & Cryptography (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-LLVM-DEAD-STORE-SECRET-LEAK` | **CWE-14 / LLVM-DSE-SECRET-ZEROIZATION-ELIMINATION** | Rust Compiler / LLVM: Secret Zeroization Stripped by Dead Store Elimination (CWE-14) | Effacement manuel d'une clé privée, seed ou secret en mémoire via .fill(0), ptr::write_bytes(..., 0, ...) ou assignation [0u8; N] sans zeroi... |

---

## 🛡️ Rust / Memory Safety & ABI (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB` | **CWE-908 / RUST-UNINIT-ASSUME-INIT-AND-STRUCT-PADDING-BLEED** | Rust ABI & Memory: MaybeUninit::uninit().assume_init() Poison UB & Struct Alignment Padding Memory Leak | Instanciation d'un tampon via MaybeUninit::uninit().assume_init() / mem::uninitialized() (poison LLVM immédiat supprimant les contrôles de b... |

---

## 🛡️ Rust / Allocator & Memory Safety (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-ALLOC-LAYOUT-ZERO-OR-ISIZE-OVERFLOW` | **CWE-680 / RUST-ALLOCATOR-ZERO-LAYOUT-AND-ISIZE-OVERFLOW-UB** | Rust Global Allocator: Zero-Sized Layout Allocation UB & from_size_align_unchecked Rounding Overflow | Appel direct à std::alloc::alloc / alloc_zeroed sans garantir layout.size() > 0 (comportement indéfini immédiat en Rust contrairement à mall... |

---

## 🛡️ Rust / ABI & FFI Soundness (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-FFI-PANIC-ACROSS-ABI-BOUNDARY` | **CWE-248 / RUST-EXTERN-C-UNWIND-ACROSS-FFI-UB** | Rust FFI / Wasm Host ABI: Uncaught Panic Unwinding Across extern \ | Fonction extern \ |

---

## 🛡️ Rust / Concurrency & LLVM Soundness (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-RUST-UNSAFECELL-CONCURRENT-MUT-NOALIAS-UB` | **CWE-362 / RUST-LLVM-NOALIAS-UNSAFECELL-MUT-UB** | Rust LLVM Codegen: Unsynchronized &mut T from UnsafeCell::get() Violating LLVM noalias | Matérialisation d'une référence exclusive &mut *cell.get() depuis un UnsafeCell dans une méthode partagée (&self) ou un type Sync sans synch... |

---

## 🛡️ Rust / OS Kernel & Storage Soundness (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🟠 **HIGH** | `CVE-RUST-MMAP-EXTERNAL-TRUNCATION-SIGBUS` | **CWE-367 / RUST-MMAP-SLICE-TOCTOU-SIGBUS-AND-UB** | Rust OS / Storage: Unlocked File Mmap::map Exposed as &[u8] (Hardware SIGBUS Crash & LLVM Jump-Table UB) | Projection d'un fichier en mémoire via memmap2::Mmap::map exposée en &[u8] sans verrouillage exclusif OS (flock / lock_exclusive) : une tron... |

---

## 🛡️ C / C++ / FFI (1 règles)

| Sévérité | Règle / Identifiant | CWE / CVE | Intitulé & Risque Technique | Description & Mécanisme de Détection |
| :--- | :--- | :--- | :--- | :--- |
| 🔴 **CRITICAL** | `CVE-CPP-UNBOUNDED-COPY-UAF` | **CWE-120 / BUFFER-OVERFLOW-FFI** | Débordement de Tampon Mémoire C/C++ (strcpy / sprintf / memcpy non borné) | Utilisation de primitives C/C++ sans contrôle de taille (strcpy, strcat, sprintf, gets, memcpy non borné) exposant la pile ou le tas à une c... |

---

