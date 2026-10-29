//! Tests de validation pour les 10 règles d'élite de sécurité (DeepSecurityRules)
//! Couvre : ZK Frozen Heart, Circom Underconstrained, DeFi Read-Only Reentrancy,
//! Account-Model Discriminator & Signer, Rust Transmute UB & Async select! cancellation,
//! Wasm Storage Bomb, L2 Calldata Fee Theft, ERC-4337 Paymaster Mempool DoS.

use security_linter::SecurityLinter;

fn assert_rule_triggers(file_path: &str, code: &str, expected_rule: &str) {
    let violations = SecurityLinter::audit_file(file_path, code, None);
    assert!(
        violations.iter().any(|v| v.rule_id == expected_rule),
        "Expected rule '{}' to trigger on '{}', but got violations: {:?}",
        expected_rule,
        file_path,
        violations.iter().map(|v| &v.rule_id).collect::<Vec<_>>()
    );
}

fn assert_rule_passes(file_path: &str, code: &str, forbidden_rule: &str) {
    let violations = SecurityLinter::audit_file(file_path, code, None);
    assert!(
        !violations.iter().any(|v| v.rule_id == forbidden_rule),
        "Forbidden rule '{}' unexpectedly triggered on '{}': {:?}",
        forbidden_rule,
        file_path,
        violations.iter().map(|v| (&v.rule_id, v.line_number)).collect::<Vec<_>>()
    );
}

#[test]
fn test_zk_weak_fiat_shamir_detection_and_safe_case() {
    let vuln_code = r#"
pub fn verify_proof(transcript: &mut MerlinTranscript, proof: &Proof) -> bool {
    let alpha = transcript.challenge_scalar(b"alpha");
    compute_pairing(alpha, proof)
}
"#;
    assert_rule_triggers("src/zk/verifier.rs", vuln_code, "CVE-ZK-WEAK-FIAT-SHAMIR");

    // Faille Frozen Heart chronologique : public_inputs est absorbé APRÈS le premier défi
    let vuln_reversed_chronology = r#"
pub fn verify_proof(transcript: &mut MerlinTranscript, proof: &Proof, public_inputs: &[Scalar]) -> bool {
    let alpha = transcript.challenge_scalar(b"alpha");
    // Trop tard ! L'attaquant a déjà calculé un challenge sans binding d'instance
    transcript.append_message(b"public_inputs", &public_inputs);
    compute_pairing(alpha, proof)
}
"#;
    assert_rule_triggers("src/zk/verifier.rs", vuln_reversed_chronology, "CVE-ZK-WEAK-FIAT-SHAMIR");

    // Cas safe : absorption AVANT le challenge
    let safe_code = r#"
pub fn verify_proof(transcript: &mut MerlinTranscript, proof: &Proof, public_inputs: &[Scalar]) -> bool {
    transcript.append_message(b"public_inputs", &public_inputs);
    let alpha = transcript.challenge_scalar(b"alpha");
    compute_pairing(alpha, proof)
}
"#;
    assert_rule_passes("src/zk/verifier.rs", safe_code, "CVE-ZK-WEAK-FIAT-SHAMIR");

    // Cas safe multi-lignes formaté par rustfmt
    let safe_multiline = r#"
pub fn verify_proof(transcript: &mut MerlinTranscript, proof: &Proof, public_inputs: &[Scalar]) -> bool {
    transcript.append_message(
        b"public_inputs",
        &public_inputs,
    );
    let alpha = transcript.challenge_scalar(b"alpha");
    compute_pairing(alpha, proof)
}
"#;
    assert_rule_passes("src/zk/verifier.rs", safe_multiline, "CVE-ZK-WEAK-FIAT-SHAMIR");
}

#[test]
fn test_zk_circom_underconstrained_signal_and_safe_case() {
    let vuln_circom = r#"
template Multiplier2() {
    signal input a;
    signal input b;
    signal output c;

    c <-- a * b;
    // Missing c === a * b constraint!
}
"#;
    assert_rule_triggers("circuits/multiplier.circom", vuln_circom, "CVE-ZK-UNDERCONSTRAINED-SIGNAL");

    let safe_circom = r#"
template Multiplier2() {
    signal input a;
    signal input b;
    signal output c;

    c <-- a * b;
    c === a * b;
}
"#;
    assert_rule_passes("circuits/multiplier.circom", safe_circom, "CVE-ZK-UNDERCONSTRAINED-SIGNAL");
}

#[test]
fn test_defi_read_only_reentrancy_and_safe_case() {
    let vuln_sol = r#"
contract LendingOracle {
    ICurvePool public pool;

    function getCollateralValue(address user) external view returns (uint256) {
        uint256 price = pool.get_virtual_price();
        return balances[user] * price / 1e18;
    }
}
"#;
    assert_rule_triggers("contracts/oracle/LendingOracle.sol", vuln_sol, "CVE-DEFI-READ-ONLY-REENTRANCY");

    let safe_sol = r#"
contract LendingOracle is ReentrancyGuard {
    ICurvePool public pool;

    function getCollateralValue(address user) external view nonReentrant returns (uint256) {
        uint256 price = pool.get_virtual_price();
        return balances[user] * price / 1e18;
    }
}
"#;
    assert_rule_passes("contracts/oracle/LendingOracle.sol", safe_sol, "CVE-DEFI-READ-ONLY-REENTRANCY");
}

#[test]
fn test_account_model_confusion_discriminator_and_safe_case() {
    let vuln_acct = r#"
pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let vault_info = &accounts[0];
    let vault_data = VaultConfig::try_from_slice(&vault_info.data.borrow())?;
    withdraw_funds(vault_data)
}
"#;
    assert_rule_triggers("programs/vault/src/processor.rs", vuln_acct, "CVE-VM-ACCOUNT-CONFUSION-DISCRIMINATOR");

    let safe_acct = r#"
pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let vault_info = &accounts[0];
    assert_eq!(&vault_info.data.borrow()[..8], VaultConfig::DISCRIMINATOR);
    let vault_data = VaultConfig::try_from_slice(&vault_info.data.borrow())?;
    withdraw_funds(vault_data)
}
"#;
    assert_rule_passes("programs/vault/src/processor.rs", safe_acct, "CVE-VM-ACCOUNT-CONFUSION-DISCRIMINATOR");
}

#[test]
fn test_account_model_missing_signer_check_and_safe_case() {
    let vuln_auth = r#"
pub fn update_admin(accounts: &[AccountInfo], new_admin: Pubkey) -> ProgramResult {
    let admin_info = &accounts[0];
    if admin_info.key == &config.admin_authority {
        config.admin_authority = new_admin;
    }
    Ok(())
}
"#;
    assert_rule_triggers("programs/vault/src/processor.rs", vuln_auth, "CVE-VM-MISSING-SIGNER-CHECK");

    let safe_auth = r#"
pub fn update_admin(accounts: &[AccountInfo], new_admin: Pubkey) -> ProgramResult {
    let admin_info = &accounts[0];
    if admin_info.key == &config.admin_authority && admin_info.is_signer {
        config.admin_authority = new_admin;
    }
    Ok(())
}
"#;
    assert_rule_passes("programs/vault/src/processor.rs", safe_auth, "CVE-VM-MISSING-SIGNER-CHECK");
}

#[test]
fn test_rust_unsound_transmute_alignment_and_safe_case() {
    let vuln_rust = r#"
pub unsafe fn parse_header(bytes: &[u8]) -> &[u64] {
    let ptr = bytes.as_ptr() as *const u64;
    std::slice::from_raw_parts(ptr, bytes.len() / 8)
}
"#;
    assert_rule_triggers("src/codec/wire.rs", vuln_rust, "CVE-RUST-UNSOUND-TRANSMUTE-ALIGNMENT");

    let safe_rust = r#"
pub fn parse_header(bytes: &[u8]) -> &[u64] {
    let (prefix, aligned, suffix) = unsafe { bytes.align_to::<u64>() };
    assert!(prefix.is_empty() && suffix.is_empty(), "Unaligned buffer");
    aligned
}
"#;
    assert_rule_passes("src/codec/wire.rs", safe_rust, "CVE-RUST-UNSOUND-TRANSMUTE-ALIGNMENT");
}

#[test]
fn test_rust_async_select_cancellation_corruption_and_safe_case() {
    let vuln_select = r#"
pub async fn send_packet(stream: &mut TcpStream, cancel: &mut Receiver<()>) {
    tokio::select! {
        _ = cancel.recv() => println!("Cancelled"),
        res = stream.write_all(&packet) => println!("Written"),
    }
}
"#;
    assert_rule_triggers("src/net/worker.rs", vuln_select, "CVE-RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION");

    let safe_select = r#"
pub async fn send_packet(tx: &mpsc::Sender<Vec<u8>>, cancel: &mut Receiver<()>) {
    tokio::select! {
        _ = cancel.recv() => println!("Cancelled"),
        _ = tx.send(packet) => println!("Enqueued"),
    }
}
"#;
    assert_rule_passes("src/net/worker.rs", safe_select, "CVE-RUST-ASYNC-SELECT-CANCELLATION-CORRUPTION");
}

#[test]
fn test_wasm_unbounded_storage_iteration_and_safe_case() {
    let vuln_cw = r#"
pub fn query_all_balances(deps: Deps) -> StdResult<Vec<Coin>> {
    let res = BALANCES.range(deps.storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
    Ok(res)
}
"#;
    assert_rule_triggers("src/contract.rs", vuln_cw, "CVE-WASM-UNBOUNDED-STORAGE-ITERATION");

    let safe_cw = r#"
pub fn query_all_balances(deps: Deps, limit: Option<u32>) -> StdResult<Vec<Coin>> {
    const DEFAULT_LIMIT: u32 = 20;
    const MAX_LIMIT: u32 = 100;
    let bound = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let res = BALANCES.range(deps.storage, None, None, Order::Ascending).take(bound).collect::<StdResult<Vec<_>>>()?;
    Ok(res)
}
"#;
    assert_rule_passes("src/contract.rs", safe_cw, "CVE-WASM-UNBOUNDED-STORAGE-ITERATION");
}

#[test]
fn test_l2_data_gas_underestimation_and_safe_case() {
    let vuln_l2 = r#"
contract RelayerArbitrum {
    function executeRelayedTx(bytes calldata data) external {
        uint256 startGas = gasleft();
        (bool ok, ) = target.call(data);
        uint256 gasUsed = startGas - gasleft();
        uint256 refund = gasUsed * tx.gasprice;
        payable(msg.sender).transfer(refund);
    }
}
"#;
    assert_rule_triggers("contracts/l2/RelayerArbitrum.sol", vuln_l2, "CVE-EVM-L2-DATA-GAS-UNDERESTIMATION");

    let safe_l2 = r#"
contract RelayerArbitrum {
    IGasPriceOracle public constant gasOracle = IGasPriceOracle(0x420000000000000000000000000000000000000F);
    function executeRelayedTx(bytes calldata data) external {
        uint256 startGas = gasleft();
        (bool ok, ) = target.call(data);
        uint256 l1Fee = gasOracle.getL1Fee(msg.data);
        uint256 refund = (startGas - gasleft()) * tx.gasprice + l1Fee;
        payable(msg.sender).transfer(refund);
    }
}
"#;
    assert_rule_passes("contracts/l2/RelayerArbitrum.sol", safe_l2, "CVE-EVM-L2-DATA-GAS-UNDERESTIMATION");
}

#[test]
fn test_erc4337_paymaster_storage_poisoning_and_safe_case() {
    let vuln_pm = r#"
contract TokenPaymaster is IPaymaster {
    uint256 public totalGasUsed;
    function validatePaymasterUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 maxCost) external returns (bytes memory context, uint256 validationData) {
        totalGasUsed += maxCost;
        return ("", 0);
    }
}
"#;
    assert_rule_triggers("contracts/paymaster/TokenPaymaster.sol", vuln_pm, "CVE-ERC4337-PAYMASTER-STORAGE-POISONING");

    let safe_pm = r#"
contract TokenPaymaster is IPaymaster {
    mapping(address => uint256) public userBalances;
    function validatePaymasterUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 maxCost) external returns (bytes memory context, uint256 validationData) {
        require(userBalances[userOp.sender] >= maxCost, "Insufficient balance");
        userBalances[userOp.sender] -= maxCost;
        return ("", 0);
    }
}
"#;
    assert_rule_passes("contracts/paymaster/TokenPaymaster.sol", safe_pm, "CVE-ERC4337-PAYMASTER-STORAGE-POISONING");
}

#[test]
fn test_zk_multi_function_scope_prove_safe_verify_vuln() {
    let multi_fn_code = r#"
pub fn prove(transcript: &mut MerlinTranscript, public_inputs: &[Scalar], witness: &[Scalar]) -> Proof {
    // prove() est sain : il absorbe les entrées publiques avant le challenge
    transcript.append_message(b"public_inputs", &public_inputs);
    let alpha = transcript.challenge_scalar(b"alpha");
    compute_quotient(alpha, witness)
}

pub fn verify(transcript: &mut MerlinTranscript, proof: &Proof) -> bool {
    // verify() est VULNÉRABLE : il a oublié d'absorber public_inputs dans sa portée !
    let alpha = transcript.challenge_scalar(b"alpha");
    check_pairing(alpha, proof)
}
"#;
    assert_rule_triggers("src/protocol.rs", multi_fn_code, "CVE-ZK-WEAK-FIAT-SHAMIR");
}

#[test]
fn test_wasm_iter_rustfmt_chained_split_lines() {
    // Cas réel rustfmt : .range() suivi de .map() puis de .take() sur des lignes distinctes
    let safe_cw_chained = r#"
pub fn query_all_balances(deps: Deps, limit: Option<u32>) -> StdResult<Vec<Coin>> {
    let bound = limit.unwrap_or(20) as usize;
    let res = BALANCES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, v)| v))
        .take(bound)
        .collect::<StdResult<Vec<_>>>()?;
    Ok(res)
}
"#;
    assert_rule_passes("src/contract.rs", safe_cw_chained, "CVE-WASM-UNBOUNDED-STORAGE-ITERATION");

    let vuln_cw_chained = r#"
pub fn query_all_balances(deps: Deps) -> StdResult<Vec<Coin>> {
    let res = BALANCES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(res)
}
"#;
    assert_rule_triggers("src/contract.rs", vuln_cw_chained, "CVE-WASM-UNBOUNDED-STORAGE-ITERATION");
}

#[test]
fn test_erc4337_postop_and_deposit_writes_are_allowed() {
    let safe_postop_contract = r#"
contract TokenPaymaster is IPaymaster {
    uint256 public totalGasUsed;

    function validatePaymasterUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 maxCost) external returns (bytes memory context, uint256 validationData) {
        // Aucune écriture d'état global ici : 100% conforme ERC-4337
        return ("", 0);
    }

    function _postOp(PostOpMode mode, bytes calldata context, uint256 actualGasCost) internal {
        // Écriture d'état globale 100% légale dans postOp (ne doit PAS déclencher d'alerte)
        totalGasUsed += actualGasCost;
    }

    function deposit() external payable {
        // Fonction d'administration légale
        globalCounter++;
    }
}
"#;
    assert_rule_passes("contracts/paymaster/TokenPaymaster.sol", safe_postop_contract, "CVE-ERC4337-PAYMASTER-STORAGE-POISONING");
}

#[test]
fn test_rust_arbitrary_file_write_traversal_and_safe_case() {
    let vuln_code = r#"
pub fn save_report(path: &str, data: &[u8]) {
    std::fs::write(path, data).unwrap();
}
"#;
    assert_rule_triggers("src/report.rs", vuln_code, "CVE-RUST-ARBITRARY-FILE-WRITE-TRAVERSAL");

    let safe_code = r#"
pub fn save_report(path: &str, data: &[u8]) {
    let p = std::path::Path::new(path).canonicalize().unwrap();
    if p.starts_with("/safe/dir") {
        std::fs::write(p, data).unwrap();
    }
}
"#;
    assert_rule_passes("src/report.rs", safe_code, "CVE-RUST-ARBITRARY-FILE-WRITE-TRAVERSAL");
}

#[test]
fn test_rust_command_injection_shell_and_safe_case() {
    let vuln_code = r#"
pub fn run_user_cmd(cmd: &str) {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .unwrap();
}
"#;
    assert_rule_triggers("src/executor.rs", vuln_code, "CVE-RUST-COMMAND-INJECTION-SHELL");

    let safe_code = r#"
pub fn run_binary(arg: &str) {
    std::process::Command::new("ls")
        .arg(arg)
        .status()
        .unwrap();
}
"#;
    assert_rule_passes("src/executor.rs", safe_code, "CVE-RUST-COMMAND-INJECTION-SHELL");
}

#[test]
fn test_rust_unsound_slice_from_raw_parts_and_safe_case() {
    let vuln_code = r#"
pub unsafe fn read_buffer(ptr: *const u8, len: usize) -> &'static [u8] {
    std::slice::from_raw_parts(ptr, len)
}
"#;
    assert_rule_triggers("src/memory.rs", vuln_code, "CVE-RUST-UNSOUND-SLICE-FROM-RAW-PARTS");

    let safe_code = r#"
pub unsafe fn read_buffer(ptr: *const u8, len: usize) -> &'static [u8] {
    assert!(!ptr.is_null());
    std::slice::from_raw_parts(ptr, len)
}
"#;
    assert_rule_passes("src/memory.rs", safe_code, "CVE-RUST-UNSOUND-SLICE-FROM-RAW-PARTS");
}

#[test]
fn test_rust_unbounded_bcs_deserialize_and_safe_case() {
    let vuln_code = r#"
pub fn decode_tx(bytes: &[u8]) -> Transaction {
    bcs::from_bytes(bytes).unwrap()
}
"#;
    assert_rule_triggers("src/decoder.rs", vuln_code, "CVE-RUST-UNBOUNDED-BCS-DESERIALIZE");

    let safe_code = r#"
pub fn decode_tx(bytes: &[u8]) -> Transaction {
    const MAX_SIZE: usize = 65536;
    assert!(bytes.len() <= MAX_SIZE);
    bcs::from_bytes(bytes).unwrap()
}
"#;
    assert_rule_passes("src/decoder.rs", safe_code, "CVE-RUST-UNBOUNDED-BCS-DESERIALIZE");
}

#[test]
fn test_rust_llvm_dead_store_secret_leak_and_safe_case() {
    let vuln_drop = r#"
pub struct SigningKey {
    secret_bytes: [u8; 32],
}
impl Drop for SigningKey {
    fn drop(&mut self) {
        self.secret_bytes.fill(0);
    }
}
"#;
    assert_rule_triggers(
        "src/crypto/keys.rs",
        vuln_drop,
        "CVE-RUST-LLVM-DEAD-STORE-SECRET-LEAK",
    );

    let safe_zeroize = r#"
use zeroize::Zeroize;
pub struct SigningKey {
    secret_bytes: [u8; 32],
}
impl Drop for SigningKey {
    fn drop(&mut self) {
        self.secret_bytes.zeroize();
    }
}
"#;
    assert_rule_passes(
        "src/crypto/keys.rs",
        safe_zeroize,
        "CVE-RUST-LLVM-DEAD-STORE-SECRET-LEAK",
    );
}

#[test]
fn test_rust_unwind_panic_double_drop_ub_and_safe_case() {
    // Cas A : set_len appelé AVANT read_exact
    let vuln_set_len = r#"
pub fn recv_frame(stream: &mut std::net::TcpStream, n: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(n.min(4096));
    unsafe { buf.set_len(n); }
    stream.read_exact(&mut buf)?;
    Ok(buf)
}
"#;
    assert_rule_triggers(
        "src/net/codec.rs",
        vuln_set_len,
        "CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB",
    );

    // Cas B : ptr::read suivi d'un appel pouvant paniquer sans ManuallyDrop
    let vuln_ptr_read = r#"
pub unsafe fn pop_and_transform<T: Clone>(ptr: *const T) -> T {
    let elem = std::ptr::read(ptr);
    let cloned = elem.clone();
    cloned
}
"#;
    assert_rule_triggers(
        "src/collections/queue.rs",
        vuln_ptr_read,
        "CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB",
    );

    let safe_set_len = r#"
pub fn recv_frame(src: &[u8], n: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(n);
    // SAFETY: src has at least n bytes and set_len is called after copy_nonoverlapping
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), buf.as_mut_ptr(), n);
        buf.set_len(n);
    }
    buf
}
"#;
    assert_rule_passes(
        "src/net/codec.rs",
        safe_set_len,
        "CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB",
    );
}

#[test]
fn test_rust_maybeuninit_padding_and_poison_ub_and_safe_case() {
    let vuln_uninit = r#"
pub fn read_raw() -> [u8; 64] {
    let buf: [u8; 64] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    buf
}
"#;
    assert_rule_triggers(
        "src/io/buffer.rs",
        vuln_uninit,
        "CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB",
    );

    let vuln_padding = r#"
#[repr(C)]
pub struct WireHeader {
    pub tag: u8,
    pub seq: u64,
}
pub fn header_bytes(hdr: &WireHeader) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            (hdr as *const WireHeader) as *const u8,
            std::mem::size_of::<WireHeader>(),
        )
    }
}
"#;
    assert_rule_triggers(
        "src/net/header.rs",
        vuln_padding,
        "CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB",
    );

    let safe_bytemuck = r#"
#[repr(C)]
#[derive(Copy, Clone, bytemuck::NoUninit)]
pub struct WireHeader {
    pub seq: u64,
}
pub fn header_bytes(hdr: &WireHeader) -> &[u8] {
    bytemuck::bytes_of(hdr)
}
"#;
    assert_rule_passes(
        "src/net/header.rs",
        safe_bytemuck,
        "CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB",
    );
}

#[test]
fn test_rust_alloc_layout_zero_or_isize_overflow_and_safe_case() {
    let vuln_alloc = r#"
pub unsafe fn raw_alloc(size: usize, align: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    std::alloc::alloc(layout)
}
"#;
    assert_rule_triggers(
        "src/arena/alloc.rs",
        vuln_alloc,
        "CVE-RUST-ALLOC-LAYOUT-ZERO-OR-ISIZE-OVERFLOW",
    );

    let safe_alloc = r#"
pub unsafe fn raw_alloc(size: usize, align: usize) -> Option<*mut u8> {
    if size == 0 || !align.is_power_of_two() || size > isize::MAX as usize - (align - 1) {
        return None;
    }
    let layout = std::alloc::Layout::from_size_align(size, align).ok()?;
    Some(std::alloc::alloc(layout))
}
"#;
    assert_rule_passes(
        "src/arena/alloc.rs",
        safe_alloc,
        "CVE-RUST-ALLOC-LAYOUT-ZERO-OR-ISIZE-OVERFLOW",
    );
}

#[test]
fn test_rust_ffi_panic_across_abi_boundary_and_safe_case() {
    let vuln_ffi = r#"
#[no_mangle]
pub extern "C" fn host_verify_sig(ptr: *const u8, len: usize) -> i32 {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let parsed = std::str::from_utf8(slice).unwrap();
    if parsed.is_empty() { 0 } else { 1 }
}
"#;
    assert_rule_triggers(
        "src/ffi/host.rs",
        vuln_ffi,
        "CVE-RUST-FFI-PANIC-ACROSS-ABI-BOUNDARY",
    );

    let safe_ffi = r#"
#[no_mangle]
pub extern "C" fn host_verify_sig(ptr: *const u8, len: usize) -> i32 {
    let res = std::panic::catch_unwind(|| {
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let parsed = std::str::from_utf8(slice).unwrap();
        if parsed.is_empty() { 0 } else { 1 }
    });
    res.unwrap_or(-1)
}
"#;
    assert_rule_passes(
        "src/ffi/host.rs",
        safe_ffi,
        "CVE-RUST-FFI-PANIC-ACROSS-ABI-BOUNDARY",
    );
}

#[test]
fn test_rust_unsafecell_concurrent_mut_noalias_ub_and_safe_case() {
    let vuln_cell = r#"
use std::cell::UnsafeCell;
pub struct SharedState {
    inner: UnsafeCell<u64>,
}
unsafe impl Sync for SharedState {}
impl SharedState {
    pub fn mutate_unchecked(&self, val: u64) {
        let r = unsafe { &mut *self.inner.get() };
        *r += val;
    }
}
"#;
    assert_rule_triggers(
        "src/state/cell.rs",
        vuln_cell,
        "CVE-RUST-UNSAFECELL-CONCURRENT-MUT-NOALIAS-UB",
    );

    let safe_cell = r#"
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
pub struct SharedState {
    locked: AtomicBool,
    inner: UnsafeCell<u64>,
}
unsafe impl Sync for SharedState {}
impl SharedState {
    pub fn mutate_locked(&self, val: u64) {
        if self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            let r = unsafe { &mut *self.inner.get() };
            *r += val;
            self.locked.store(false, Ordering::Release);
        }
    }
}
"#;
    assert_rule_passes(
        "src/state/cell.rs",
        safe_cell,
        "CVE-RUST-UNSAFECELL-CONCURRENT-MUT-NOALIAS-UB",
    );
}

#[test]
fn test_rust_mmap_external_truncation_sigbus_and_safe_case() {
    let vuln_mmap = r#"
pub fn open_snapshot(file: &std::fs::File) -> memmap2::Mmap {
    unsafe { memmap2::Mmap::map(file).unwrap() }
}
"#;
    assert_rule_triggers(
        "src/storage/mmap_store.rs",
        vuln_mmap,
        "CVE-RUST-MMAP-EXTERNAL-TRUNCATION-SIGBUS",
    );

    let safe_mmap = r#"
pub fn open_snapshot(file: &std::fs::File) -> memmap2::Mmap {
    file.lock_exclusive().unwrap();
    // SAFETY: Exclusive OS file lock acquired prior to mapping.
    unsafe { memmap2::Mmap::map(file).unwrap() }
}
"#;
    assert_rule_passes(
        "src/storage/mmap_store.rs",
        safe_mmap,
        "CVE-RUST-MMAP-EXTERNAL-TRUNCATION-SIGBUS",
    );
}

#[test]
fn test_global_cfg_test_cutoff_blocks_deep_rules_in_unit_tests() {
    let prod_with_inline_tests = r#"
pub fn safe_production_fn(x: u64) -> u64 {
    x.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unsafe_helpers() {
        let buf: [u8; 32] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let _ = bcs::from_bytes::<u64>(&buf).unwrap();
    }
}
"#;
    let violations = SecurityLinter::audit_file("src/runtime/executor.rs", prod_with_inline_tests, None);
    assert!(
        violations.is_empty(),
        "Inline #[cfg(test)] block must be stripped before DeepSecurityRules runs, got: {:?}",
        violations
    );
}
