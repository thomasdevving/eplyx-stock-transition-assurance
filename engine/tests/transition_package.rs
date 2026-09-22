use eplyx_lifecycle_impact::conversion::package::{self, load};
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

#[test]
fn schema_adapter_address_and_status_claims_fail_closed() {
    for (label, path, replacement) in [
        ("schema", "schemaVersion", json!(2)),
        ("adapter", "adapter", json!("arbitrary_program")),
        ("mint", "sourceMint", json!("bad address")),
        ("proof", "officialTransition", json!("Proven")),
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

    let config = PackageDir::new("config-hash");
    fs::write(config.0.join("config.json"), b"{}").unwrap();
    assert!(config.error().contains("config SHA-256 mismatch"));
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
