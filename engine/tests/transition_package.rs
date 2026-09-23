use eplyx_lifecycle_impact::conversion::{
    current as conversion, demo,
    package::{self, load},
};
use eplyx_lifecycle_impact::executor::LoadedProgram;
use eplyx_lifecycle_impact::lifecycle::{current as wallet, decode::MintConfig, RpcEvidence};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

struct PackageDir(PathBuf);
impl PackageDir {
    fn new(label: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let directory = root.join("target").join(format!(
            "transition-package-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        let example = root.join("examples/transitions/demo-fixed-ratio");
        for file in ["eplyx.json", "config.json", "program.so"] {
            fs::copy(example.join(file), directory.join(file)).unwrap();
        }
        Self(directory)
    }
    fn manifest(&self) -> Value {
        serde_json::from_slice(&fs::read(self.0.join("eplyx.json")).unwrap()).unwrap()
    }
    fn set_manifest(&self, value: Value) {
        fs::write(
            self.0.join("eplyx.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
    }
    fn error(&self) -> String {
        load(&self.0).err().unwrap().to_string()
    }
}
impl Drop for PackageDir {
    fn drop(&mut self) {
        let target = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target");
        assert!(self.0.starts_with(target));
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn valid_package_has_deterministic_identity_and_proposed_origin() {
    let p = PackageDir::new("valid");
    let a = load(&p.0).unwrap();
    let b = load(&p.0).unwrap();
    assert_eq!(a.transition_package_sha256, b.transition_package_sha256);
    assert_eq!(a.program_sha256, a.manifest.candidate_program.sha256);
    let plan = a.conversion_plan().unwrap();
    assert_eq!(
        plan.provenance,
        eplyx_lifecycle_impact::conversion::PlanProvenance::OperatorSupplied
    );
    assert_eq!(plan.source_mint, a.manifest.source_mint);
    assert_eq!(plan.replacement_mint, a.manifest.replacement_mint);
}

fn invariant_manifest(p: &PackageDir) -> Value {
    let mut manifest = p.manifest();
    manifest["schemaVersion"] = json!(2);
    manifest["invariantSchemaVersion"] = json!(1);
    manifest["invariants"] = json!([
        {"type": "conversion_output_matches", "severity": "blocking"},
        {"type": "no_selected_case_failed", "severity": "blocking"},
        {"type": "authority_model_supported", "severity": "warning"}
    ]);
    manifest
}

#[test]
fn invariant_schema_rejects_unknown_malformed_and_duplicate_definitions() {
    for (label, definition) in [
        (
            "unknown",
            json!({"type":"operator_script", "severity":"blocking"}),
        ),
        (
            "missing-severity",
            json!({"type":"conversion_output_matches"}),
        ),
        (
            "extra-field",
            json!({"type":"conversion_output_matches", "severity":"blocking", "status":"Satisfied"}),
        ),
        (
            "arbitrary-path",
            json!({"type":"required_path_available", "severity":"blocking", "path":"/any/json/path"}),
        ),
    ] {
        let p = PackageDir::new(label);
        let mut manifest = invariant_manifest(&p);
        manifest["invariants"] = json!([definition]);
        p.set_manifest(manifest);
        assert!(p.error().contains("invalid package manifest"), "{label}");
    }
    let p = PackageDir::new("duplicate-invariant");
    let mut manifest = invariant_manifest(&p);
    manifest["invariants"] = json!([
        {"type":"conversion_output_matches", "severity":"blocking"},
        {"type":"conversion_output_matches", "severity":"warning"}
    ]);
    p.set_manifest(manifest);
    assert!(p.error().contains("duplicate invariant"));
}

#[test]
fn invariant_identity_binds_type_severity_config_and_is_order_independent() {
    let a = PackageDir::new("invariant-identity-a");
    let original = invariant_manifest(&a);
    a.set_manifest(original.clone());
    let original_hash = load(&a.0).unwrap().transition_package_sha256;

    let reordered = PackageDir::new("invariant-order");
    let mut reversed = original.clone();
    reversed["invariants"].as_array_mut().unwrap().reverse();
    reordered.set_manifest(reversed);
    assert_eq!(
        original_hash,
        load(&reordered.0).unwrap().transition_package_sha256
    );

    for (label, changed) in [
        (
            "severity",
            json!({"type":"authority_model_supported", "severity":"blocking"}),
        ),
        (
            "type",
            json!({"type":"no_positive_balance_stranded", "severity":"warning"}),
        ),
        (
            "config",
            json!({"type":"required_path_available", "severity":"warning", "path":"OfficialTransition"}),
        ),
    ] {
        let p = PackageDir::new(label);
        let mut manifest = original.clone();
        manifest["invariants"][2] = changed;
        p.set_manifest(manifest);
        assert_ne!(
            original_hash,
            load(&p.0).unwrap().transition_package_sha256,
            "{label}"
        );
    }
    let old = PackageDir::new("schema-one-old-identity");
    assert_ne!(
        original_hash,
        load(&old.0).unwrap().transition_package_sha256
    );
}

#[test]
fn schema_adapter_address_and_status_claims_fail_closed() {
    for (label, path, replacement) in [
        ("schema", "schemaVersion", json!(2)),
        ("adapter", "adapter", json!("arbitrary_program")),
        ("mint", "sourceMint", json!("bad address")),
        ("proof", "officialTransition", json!("Proven")),
        ("rpc", "rpcEndpoint", json!("https://example.invalid")),
        ("command", "hostCommand", json!("echo unsafe")),
        ("transaction", "transactionBytes", json!([1, 2, 3])),
        ("readiness", "candidatePlanReadiness", json!("Ready")),
        ("gate", "gateOutcome", json!("Pass")),
    ] {
        let p = PackageDir::new(label);
        let mut manifest = p.manifest();
        manifest[path] = replacement;
        p.set_manifest(manifest);
        assert!(!p.error().is_empty(), "{label}");
    }
    let p = PackageDir::new("duplicate");
    let mut raw = fs::read_to_string(p.0.join("eplyx.json")).unwrap();
    raw = raw.replacen(
        "\"schemaVersion\": 1,",
        "\"schemaVersion\": 1, \"schemaVersion\": 1,",
        1,
    );
    fs::write(p.0.join("eplyx.json"), raw).unwrap();
    assert!(p.error().contains("invalid package manifest"));
    let p = PackageDir::new("alternate-binary");
    let mut manifest = p.manifest();
    manifest["candidateProgram"]["alternateArtifact"] = json!("other.so");
    p.set_manifest(manifest);
    assert!(p.error().contains("invalid package manifest"));
}

#[test]
fn config_and_integer_terms_reject_unknown_or_ambiguous_values() {
    let p = PackageDir::new("config-claim");
    let mut config: Value =
        serde_json::from_slice(&fs::read(p.0.join("config.json")).unwrap()).unwrap();
    config["conversionStatus"] = json!("Proven");
    let bytes = serde_json::to_vec(&config).unwrap();
    fs::write(p.0.join("config.json"), &bytes).unwrap();
    let mut manifest = p.manifest();
    manifest["configSha256"] = json!(eplyx_lifecycle_impact::lifecycle::exposure::sha256(&bytes));
    p.set_manifest(manifest);
    assert!(p.error().contains("invalid package config"));

    for (label, key, value) in [
        (
            "config-rpc",
            "rpcEndpoint",
            json!("https://example.invalid"),
        ),
        ("config-command", "hostCommand", json!("echo unsafe")),
        ("config-transaction", "transactionBytes", json!([1, 2, 3])),
        ("config-proof", "proofStatus", json!("Proven")),
        ("config-gate", "gateOutcome", json!("Pass")),
        (
            "config-binary",
            "alternateCandidateBinary",
            json!("../other.so"),
        ),
    ] {
        let p = PackageDir::new(label);
        let mut config: Value =
            serde_json::from_slice(&fs::read(p.0.join("config.json")).unwrap()).unwrap();
        config[key] = value;
        let bytes = serde_json::to_vec(&config).unwrap();
        fs::write(p.0.join("config.json"), &bytes).unwrap();
        let mut manifest = p.manifest();
        manifest["configSha256"] =
            json!(eplyx_lifecycle_impact::lifecycle::exposure::sha256(&bytes));
        p.set_manifest(manifest);
        assert!(p.error().contains("invalid package config"), "{label}");
    }

    for (label, field, value) in [
        ("zero-ratio", "denominator", "0"),
        ("noncanonical-ratio", "numerator", "01"),
        ("overflow-ratio", "numerator", "18446744073709551616"),
    ] {
        let p = PackageDir::new(label);
        let mut manifest = p.manifest();
        manifest["terms"][field] = json!(value);
        p.set_manifest(manifest);
        assert!(!p.error().is_empty(), "{label}");
    }
}

#[test]
fn artifact_paths_and_binary_are_bounded_to_sbf_inside_package() {
    let p = PackageDir::new("traversal");
    let mut manifest = p.manifest();
    manifest["candidateProgram"]["artifact"] = json!("../program.so");
    p.set_manifest(manifest);
    assert!(p.error().contains("traversal"));

    let p = PackageDir::new("missing");
    fs::remove_file(p.0.join("program.so")).unwrap();
    assert!(!p.error().is_empty());

    let p = PackageDir::new("native");
    let mut native = fs::read(p.0.join("program.so")).unwrap();
    native[18] = 62;
    native[19] = 0;
    fs::write(p.0.join("program.so"), &native).unwrap();
    let mut manifest = p.manifest();
    manifest["candidateProgram"]["sha256"] =
        json!(eplyx_lifecycle_impact::lifecycle::exposure::sha256(&native));
    p.set_manifest(manifest);
    assert!(p.error().contains("SBF/BPF"));

    let p = PackageDir::new("oversized");
    fs::write(
        p.0.join("program.so"),
        vec![0_u8; (package::MAX_PROGRAM_BYTES + 1) as usize],
    )
    .unwrap();
    assert!(p.error().contains("size bound"));
}

#[test]
fn symlink_escape_is_rejected_when_the_platform_allows_symlinks() {
    let p = PackageDir::new("symlink");
    let outside = p.0.parent().unwrap().join("outside-program.so");
    fs::copy(p.0.join("program.so"), &outside).unwrap();
    fs::remove_file(p.0.join("program.so")).unwrap();
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_file(&outside, p.0.join("program.so"));
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(&outside, p.0.join("program.so"));
    if linked.is_ok() {
        assert!(p.error().contains("escapes package root"));
    }
    fs::remove_file(outside).unwrap();
}

#[test]
fn changing_terms_program_or_config_changes_or_invalidates_identity() {
    let base = PackageDir::new("identity-base");
    let original = load(&base.0).unwrap().transition_package_sha256;
    let ratio = PackageDir::new("identity-ratio");
    let mut manifest = ratio.manifest();
    manifest["terms"]["numerator"] = json!("3");
    ratio.set_manifest(manifest);
    assert_ne!(original, load(&ratio.0).unwrap().transition_package_sha256);

    let program = PackageDir::new("program-hash");
    let mut bytes = fs::read(program.0.join("program.so")).unwrap();
    bytes[64] ^= 1;
    fs::write(program.0.join("program.so"), &bytes).unwrap();
    assert!(program.error().contains("SHA-256 mismatch"));
    let mut manifest = program.manifest();
    manifest["candidateProgram"]["sha256"] =
        json!(eplyx_lifecycle_impact::lifecycle::exposure::sha256(&bytes));
    program.set_manifest(manifest);
    assert_ne!(
        original,
        load(&program.0).unwrap().transition_package_sha256
    );
    assert_ne!(
        load(&base.0).unwrap().program_sha256,
        load(&program.0).unwrap().program_sha256
    );

    let config = PackageDir::new("config-hash");
    fs::write(config.0.join("config.json"), b"{}").unwrap();
    assert!(config.error().contains("config SHA-256 mismatch"));
    let bytes = fs::read(config.0.join("config.json")).unwrap();
    let mut manifest = config.manifest();
    manifest["configSha256"] = json!(eplyx_lifecycle_impact::lifecycle::exposure::sha256(&bytes));
    config.set_manifest(manifest);
    assert!(config.error().contains("invalid package config"));

    let valid_config = PackageDir::new("valid-config-change");
    let mut config_value: Value =
        serde_json::from_slice(&fs::read(valid_config.0.join("config.json")).unwrap()).unwrap();
    config_value["reserveFundedReplacementRaw"] = json!("42");
    let config_bytes = serde_json::to_vec(&config_value).unwrap();
    fs::write(valid_config.0.join("config.json"), &config_bytes).unwrap();
    let mut manifest = valid_config.manifest();
    manifest["configSha256"] = json!(eplyx_lifecycle_impact::lifecycle::exposure::sha256(
        &config_bytes
    ));
    valid_config.set_manifest(manifest);
    assert_ne!(
        original,
        load(&valid_config.0).unwrap().transition_package_sha256
    );

    for (label, key, value) in [
        ("rounding", "rounding", json!("ceiling")),
        ("fee", "feeBps", json!(7)),
    ] {
        let p = PackageDir::new(label);
        let mut manifest = p.manifest();
        manifest["terms"][key] = value;
        p.set_manifest(manifest);
        assert_ne!(original, load(&p.0).unwrap().transition_package_sha256);
    }

    for (label, key, value) in [
        (
            "source-mint",
            "sourceMint",
            "PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF",
        ),
        (
            "replacement-mint",
            "replacementMint",
            "PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF",
        ),
    ] {
        let p = PackageDir::new(label);
        let mut manifest = p.manifest();
        manifest[key] = json!(value);
        p.set_manifest(manifest);
        assert_ne!(original, load(&p.0).unwrap().transition_package_sha256);
    }
}

#[test]
fn packaged_candidate_bytes_are_the_only_vm_program() {
    let p = PackageDir::new("exact-vm-program");
    let original = load(&p.0).unwrap();
    let mut changed = original.program.clone();
    changed[64] ^= 1;
    fs::write(p.0.join("program.so"), &changed).unwrap();
    let mut manifest = p.manifest();
    manifest["candidateProgram"]["sha256"] = json!(
        eplyx_lifecycle_impact::lifecycle::exposure::sha256(&changed)
    );
    p.set_manifest(manifest);
    let mutated = load(&p.0).unwrap();
    assert_ne!(mutated.program_sha256, original.program_sha256);
    assert_ne!(
        mutated.transition_package_sha256,
        original.transition_package_sha256
    );
    let stale_repository_fallback = [LoadedProgram {
        program_id: demo::PROGRAM_ID.parse().unwrap(),
        loader: demo::LOADER.parse().unwrap(),
        bytes: original.program,
    }];
    assert!(demo::assert_candidate_program_identity(
        &stale_repository_fallback,
        &mutated.program,
        &mutated.program_sha256,
    )
    .unwrap_err()
    .to_string()
    .contains("bytes differ from the validated package"));
    let exact_package = [LoadedProgram {
        program_id: demo::PROGRAM_ID.parse().unwrap(),
        loader: demo::LOADER.parse().unwrap(),
        bytes: mutated.program.clone(),
    }];
    demo::assert_candidate_program_identity(
        &exact_package,
        &mutated.program,
        &mutated.program_sha256,
    )
    .unwrap();

    // Rebuild an actual saved production fixture with the altered package
    // binary. This catches a build path that silently substitutes repository
    // bytes after package validation, not merely a broken helper.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let capture: Value = serde_json::from_slice(
        &fs::read(root.join("reports/milestone8-healthy-worker/conversion.capture.json")).unwrap(),
    )
    .unwrap();
    let plan: eplyx_lifecycle_impact::conversion::ConversionPlan =
        serde_json::from_value(capture["plan"].clone()).unwrap();
    let wallet_capture: wallet::Capture =
        serde_json::from_str(capture["wallet_capture"].as_str().unwrap()).unwrap();
    let observed = wallet::evaluate(&wallet_capture).unwrap();
    let row = observed["wallet_observation"]["token_accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["address"] == plan.source_account)
        .unwrap();
    let mint: MintConfig = serde_json::from_value(observed["mint"].clone()).unwrap();
    let scope = conversion::validate(capture["wallet_capture"].as_str().unwrap(), &plan).unwrap();
    let context = demo::ConversionContext {
        genesis_hash: observed["acquisition"]["genesis_hash"]
            .as_str()
            .unwrap()
            .into(),
        minimum_slot: row["slot"].as_u64().unwrap(),
        owner: original.config.public_owner,
        source_program: mint.token_program,
        source_decimals: mint.decimals,
        amount: scope["amount_raw"].as_str().unwrap().parse().unwrap(),
    };
    let evidence: Vec<RpcEvidence> = capture["observations"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(id, record)| RpcEvidence {
            id,
            method: record["method"].as_str().unwrap().into(),
            params: record["params"].clone(),
            result: record["result"].clone(),
        })
        .collect();
    let built = demo::build(
        &plan,
        capture["plan_sha256"].as_str().unwrap(),
        &context,
        &evidence,
        &mutated.program,
    )
    .unwrap();
    demo::assert_candidate_program_identity(
        &built.plan.programs,
        &mutated.program,
        &mutated.program_sha256,
    )
    .unwrap();
    let execution = eplyx_lifecycle_impact::executor::execute_probe_message(
        &built.plan.accounts,
        &built.plan.watch,
        built.plan.clock,
        &built.plan.programs,
        built.plan.message,
    );
    if let Err(error) = execution {
        assert!(
            error
                .to_string()
                .contains("cannot load captured executable"),
            "changed binary must execute or be rejected by the VM loader: {error:#}"
        );
    }
}

#[test]
fn candidate_program_digest_mismatch_is_rejected_before_vm() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let package = load(&root.join("examples/transitions/demo-fixed-ratio")).unwrap();
    let programs = [LoadedProgram {
        program_id: demo::PROGRAM_ID.parse().unwrap(),
        loader: demo::LOADER.parse().unwrap(),
        bytes: package.program.clone(),
    }];
    assert!(
        demo::assert_candidate_program_identity(&programs, &package.program, &"0".repeat(64))
            .unwrap_err()
            .to_string()
            .contains("candidate program digest mismatch")
    );
}

#[test]
fn second_package_has_distinct_asset_and_terms() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let first = load(&root.join("examples/transitions/demo-fixed-ratio")).unwrap();
    let second = load(&root.join("examples/transitions/demo-second-asset")).unwrap();
    assert_ne!(
        first.transition_package_sha256,
        second.transition_package_sha256
    );
    assert_ne!(first.manifest.source_mint, second.manifest.source_mint);
    assert_ne!(
        first.conversion_plan().unwrap().terms,
        second.conversion_plan().unwrap().terms
    );
}
