//! Digest-bound public observations; this adapter performs no network calls.
use super::*;
use crate::{
    expansion::{canonical, digest, load},
    lifecycle::{
        decode, decode::MintConfig, exposure::sha256, policy::LifecycleScenario, LifecycleSnapshot,
    },
    resolution::{
        self, ArtifactRef, DiscoveryManifest, DiscoverySource, EvidenceKind, LifecycleResolution,
    },
};
use anyhow::Context;
use serde_json::{json, Value};
use solana_address::Address;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssuerSemantics {
    pub holder_action: String,
    pub successor_reference: String,
    pub conversion_description: String,
    pub kyc_conversion_claim: Option<String>,
    pub deadline_claim: Option<String>,
    pub conversion_ratio: Option<String>,
    pub onchain_claim: bool,
    pub infrastructure_required: Option<bool>,
    pub evidence_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchLimits {
    pub maximum_signature_queries: usize,
    pub maximum_returned_signatures: usize,
    pub maximum_unique_transaction_samples: usize,
    pub maximum_transaction_retries: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchManifest {
    pub schema_version: u32,
    pub entity_id: String,
    pub source_asset: String,
    pub destination_asset: String,
    pub prior_resolution: ArtifactRef,
    pub prior_discovery: ArtifactRef,
    pub evidence_bundle: ArtifactRef,
    pub coverage: ArtifactRef,
    pub archives: Vec<ArtifactRef>,
    pub supplementary_artifacts: Vec<ArtifactRef>,
    pub external_sources: Vec<DiscoverySource>,
    pub issuer_semantics: IssuerSemantics,
    pub limits: ResearchLimits,
    pub stop_condition: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEvidence {
    pub artifact: ArtifactRef,
    pub pointer: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MintVerification {
    pub mint: String,
    pub slot: u64,
    pub runtime_owner: String,
    pub raw_data_sha256: String,
    pub configuration: MintConfig,
    pub evidence: RawEvidence,
    pub creation_slot_established: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchQuery {
    pub address: String,
    pub requested_limit: u64,
    pub returned: usize,
    pub newest_slot: Option<u64>,
    pub oldest_slot: Option<u64>,
    pub signatures: Vec<String>,
    pub evidence: RawEvidence,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgramVerification {
    pub program_id: String,
    pub executable: Option<bool>,
    pub runtime_owner: Option<String>,
    pub evidence: Option<RawEvidence>,
    pub programdata_address: Option<String>,
    pub programdata_evidence: Option<RawEvidence>,
    pub deployment_slot: Option<u64>,
    pub upgrade_authority: Option<String>,
    pub elf_input_sha256: Option<String>,
    pub loader_link_verified: bool,
    pub official_transition_identity_established: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTokenDelta {
    pub account: String,
    pub mint: String,
    pub observed_owner: Option<String>,
    pub before_raw: String,
    pub after_raw: String,
    pub delta_raw: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountClassification {
    ObservedSigner,
    PublicProgram,
    ProgramDerived,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransactionObservation {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub transaction_error: Value,
    pub evidence: RawEvidence,
    pub account_list: Value,
    pub account_classifications: BTreeMap<String, AccountClassification>,
    pub signer_list: Vec<String>,
    pub programs: Vec<String>,
    pub outer_instructions: Value,
    pub inner_instructions: Value,
    pub logs: Value,
    pub token_deltas: Vec<ObservedTokenDelta>,
    pub instruction_types: Vec<String>,
    pub touches_source: bool,
    pub touches_successor: bool,
    pub same_owner_source_debit_successor_credit: bool,
    pub interpretation: String,
    pub official_transition_identity_established: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountRelationship {
    pub address: String,
    pub slot: u64,
    pub runtime_owner: Option<String>,
    pub executable: Option<bool>,
    pub raw_data_sha256: Option<String>,
    pub evidence: RawEvidence,
    pub verified_pdas: Vec<Value>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialTransitionReport {
    pub schema_version: u32,
    pub entity_id: String,
    pub retained_owner: String,
    pub observed_balance_raw: String,
    pub research_manifest_sha256: String,
    pub issuer_semantics: IssuerSemantics,
    pub external_sources: Vec<DiscoverySource>,
    pub source_verification: MintVerification,
    pub successor_verification: MintVerification,
    pub shared_base_authorities: Vec<String>,
    pub searches: Vec<SearchQuery>,
    pub transaction_requests: usize,
    pub transaction_retry_requests: usize,
    pub unique_transaction_samples: usize,
    pub available_transaction_samples: usize,
    pub rpc_errors: Vec<RawEvidence>,
    pub transactions: Vec<TransactionObservation>,
    pub programs: Vec<ProgramVerification>,
    pub account_relationships: Vec<AccountRelationship>,
    pub candidate_mechanisms: Vec<OfficialTransitionMechanism>,
    pub selected_mechanism: OfficialTransitionMechanism,
    pub assessment: TransitionAssessment,
    pub exact_tested_input_raw: Option<String>,
    pub stop_condition: String,
    pub limitations: Vec<String>,
    pub updated_discovery: DiscoveryManifest,
    pub updated_resolution: LifecycleResolution,
}
impl OfficialTransitionReport {
    pub fn to_json(&self) -> Result<String> {
        canonical(self)
    }
    pub fn render_text(&self) -> String {
        format!("Official transition investigation\nEntity: {}\nRetained owner: {}\nObserved source amount: {} raw\nSuccessor: {}\nSuccessor bank slot: {}\nStatus: {:?}\n{}\nSearches: {}; returned signatures: {}; unique transaction samples: {}; available: {}; retries: {}\nIndependent execution supported: {}\nExecution attempted: {}\nTransition non-existence is not established. No mainnet transaction.\n\n{}", self.entity_id,self.retained_owner,self.observed_balance_raw,self.successor_verification.mint,self.successor_verification.slot,self.assessment.status,self.assessment.reason,self.searches.len(),self.searches.iter().map(|q|q.returned).sum::<usize>(),self.unique_transaction_samples,self.available_transaction_samples,self.transaction_retry_requests,self.assessment.independent_execution_supported,self.assessment.execution_attempted,self.updated_resolution.render_text())
    }
}
struct AccountRecord {
    raw: Value,
    slot: u64,
    evidence: RawEvidence,
}
fn ref_at(artifact: &ArtifactRef, pointer: String) -> RawEvidence {
    RawEvidence {
        artifact: artifact.clone(),
        pointer,
    }
}
fn authorities(m: &MintConfig) -> BTreeSet<String> {
    [m.mint_authority.clone(), m.freeze_authority.clone()]
        .into_iter()
        .flatten()
        .collect()
}
fn mint(address: &str, accounts: &BTreeMap<String, AccountRecord>) -> Result<MintVerification> {
    let a = accounts
        .get(address)
        .context("required current mint bytes absent")?;
    Ok(MintVerification {
        mint: address.into(),
        slot: a.slot,
        runtime_owner: a.raw["owner"].as_str().context("mint owner absent")?.into(),
        raw_data_sha256: sha256(&decode::raw_account_bytes(&a.raw)?),
        configuration: decode::decode_mint(&a.raw)?,
        evidence: a.evidence.clone(),
        creation_slot_established: None,
    })
}
fn program(id: &str, accounts: &BTreeMap<String, AccountRecord>) -> Result<ProgramVerification> {
    let mut p = ProgramVerification {
        program_id: id.into(),
        executable: None,
        runtime_owner: None,
        evidence: None,
        programdata_address: None,
        programdata_evidence: None,
        deployment_slot: None,
        upgrade_authority: None,
        elf_input_sha256: None,
        loader_link_verified: false,
        official_transition_identity_established: false,
    };
    let Some(a) = accounts.get(id).filter(|a| !a.raw.is_null()) else {
        return Ok(p);
    };
    p.executable = a.raw["executable"].as_bool();
    p.runtime_owner = a.raw["owner"].as_str().map(str::to_string);
    p.evidence = Some(a.evidence.clone());
    if p.executable == Some(true)
        && p.runtime_owner.as_deref() == Some("BPFLoaderUpgradeab1e11111111111111111111111")
    {
        let bytes = decode::raw_account_bytes(&a.raw)?;
        ensure!(
            bytes.len() == 36 && bytes[..4] == 2u32.to_le_bytes(),
            "invalid executable loader header"
        );
        let address = Address::new_from_array(bytes[4..36].try_into()?).to_string();
        p.programdata_address = Some(address.clone());
        if let Some(data) = accounts.get(&address) {
            ensure!(
                data.raw["owner"] == a.raw["owner"] && data.raw["executable"] == false,
                "invalid ProgramData owner/flags"
            );
            let bytes = decode::raw_account_bytes(&data.raw)?;
            ensure!(
                bytes.len() >= 45
                    && bytes[..4] == 3u32.to_le_bytes()
                    && bytes[12] <= 1
                    && bytes[45..].starts_with(b"\x7fELF"),
                "invalid captured ProgramData"
            );
            p.deployment_slot = Some(u64::from_le_bytes(bytes[4..12].try_into()?));
            if bytes[12] == 1 {
                p.upgrade_authority =
                    Some(Address::new_from_array(bytes[13..45].try_into()?).to_string());
            }
            p.elf_input_sha256 = Some(sha256(&bytes[45..]));
            p.programdata_evidence = Some(data.evidence.clone());
            p.loader_link_verified = true;
        }
    }
    Ok(p)
}
fn observation(
    sig: &str,
    tx: &Value,
    evidence: RawEvidence,
    source: &str,
    destination: &str,
    programs: &BTreeMap<String, ProgramVerification>,
    derived: &BTreeSet<String>,
) -> Result<TransactionObservation> {
    let message = &tx["transaction"]["message"];
    let meta = &tx["meta"];
    let keys = message["accountKeys"]
        .as_array()
        .context("parsed transaction account list missing")?;
    let mut before = BTreeMap::new();
    let mut after = BTreeMap::new();
    for (field, map) in [
        ("preTokenBalances", &mut before),
        ("postTokenBalances", &mut after),
    ] {
        for b in meta[field].as_array().into_iter().flatten() {
            let index = b["accountIndex"].as_u64().context("token index missing")? as usize;
            ensure!(index < keys.len(), "token account index out of bounds");
            let mint = b["mint"]
                .as_str()
                .context("token mint missing")?
                .to_string();
            ensure!(
                map.insert((index, mint), b).is_none(),
                "duplicate token balance"
            );
        }
    }
    let mut deltas = Vec::new();
    for (index, mint) in before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        let key = (index, mint.clone());
        let b = before.get(&key);
        let a = after.get(&key);
        let amount = |v: Option<&&Value>| -> Result<u64> {
            Ok(v.map(|v| {
                v["uiTokenAmount"]["amount"]
                    .as_str()
                    .context("raw token amount absent")
            })
            .transpose()?
            .unwrap_or("0")
            .parse()?)
        };
        let pre = amount(b)?;
        let post = amount(a)?;
        deltas.push(ObservedTokenDelta {
            account: keys[index]["pubkey"]
                .as_str()
                .context("account key missing")?
                .into(),
            mint,
            observed_owner: b
                .or(a)
                .and_then(|v| v["owner"].as_str())
                .map(str::to_string),
            before_raw: pre.to_string(),
            after_raw: post.to_string(),
            delta_raw: (i128::from(post) - i128::from(pre)).to_string(),
        });
    }
    let touches_source = deltas.iter().any(|d| d.mint == source);
    let touches_successor = deltas.iter().any(|d| d.mint == destination);
    let paired = deltas
        .iter()
        .filter(|d| d.mint == source && d.delta_raw.starts_with('-'))
        .any(|s| {
            s.observed_owner.is_some()
                && deltas.iter().any(|d| {
                    d.mint == destination
                        && !d.delta_raw.starts_with('-')
                        && d.delta_raw != "0"
                        && d.observed_owner == s.observed_owner
                })
        });
    let outer = message["instructions"].clone();
    let inner = meta["innerInstructions"].clone();
    let mut ids = BTreeSet::new();
    let mut types = BTreeSet::new();
    for ix in outer.as_array().into_iter().flatten().chain(
        inner
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|i| i["instructions"].as_array().into_iter().flatten()),
    ) {
        ids.insert(
            ix["programId"]
                .as_str()
                .context("instruction program missing")?
                .to_string(),
        );
        if let Some(t) = ix["parsed"]["type"].as_str() {
            types.insert(t.to_string());
        }
    }
    let classification = keys
        .iter()
        .map(|k| {
            let id = k["pubkey"].as_str().unwrap().to_string();
            let c = if programs
                .get(&id)
                .is_some_and(|p| p.executable == Some(true))
            {
                AccountClassification::PublicProgram
            } else if derived.contains(&id) {
                AccountClassification::ProgramDerived
            } else if k["signer"] == true {
                AccountClassification::ObservedSigner
            } else {
                AccountClassification::Unknown
            };
            (id, c)
        })
        .collect();
    let interpretation = if touches_source && touches_successor {
        "Both assets appear in a routed token/market transaction; mint co-occurrence and token deltas alone do not identify an issuer-defined transition."
    } else if touches_source && types.contains("burn") {
        "Source burn/account closure is observed; no successor credit or official exchange is established."
    } else if touches_successor && types.contains("mintTo") {
        "Successor administrative minting is observed; no source-side exchange or holder eligibility is established."
    } else if types.contains("withdrawWithheldTokensFromMint") {
        "Administrative withheld-fee withdrawal, not a holder conversion."
    } else {
        "Public transaction observation without independently established official conversion identity."
    };
    Ok(TransactionObservation {
        signature: sig.into(),
        slot: tx["slot"].as_u64().context("transaction slot absent")?,
        block_time: tx["blockTime"].as_i64(),
        transaction_error: meta["err"].clone(),
        evidence,
        account_list: message["accountKeys"].clone(),
        account_classifications: classification,
        signer_list: keys
            .iter()
            .filter(|k| k["signer"] == true)
            .filter_map(|k| k["pubkey"].as_str().map(str::to_string))
            .collect(),
        programs: ids.into_iter().collect(),
        outer_instructions: outer,
        inner_instructions: inner,
        logs: meta["logMessages"].clone(),
        token_deltas: deltas,
        instruction_types: types.into_iter().collect(),
        touches_source,
        touches_successor,
        same_owner_source_debit_successor_credit: paired,
        interpretation: interpretation.into(),
        official_transition_identity_established: false,
    })
}

pub fn investigate(
    manifest_path: &Path,
    snapshot: &LifecycleSnapshot,
    scenario: &LifecycleScenario,
    requested_entity: &str,
) -> Result<OfficialTransitionReport> {
    let manifest: ResearchManifest = load(manifest_path)?;
    ensure!(
        manifest.schema_version == 1,
        "unsupported transition research schema"
    );
    let base = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let entity = if requested_entity.starts_with("solana-token-account:") {
        requested_entity.to_string()
    } else {
        format!("solana-token-account:{requested_entity}")
    };
    ensure!(
        entity == manifest.entity_id,
        "research does not inherit to another entity"
    );
    let prior: LifecycleResolution =
        serde_json::from_slice(&manifest.prior_resolution.read(base)?)?;
    ensure!(
        prior.entity_id == entity
            && prior.asset_mint == manifest.source_asset
            && prior.snapshot_sha256 == digest(snapshot)?
            && prior.scenario_sha256 == scenario.sha256()?,
        "research entity/population/policy mismatch"
    );
    manifest.prior_discovery.read(base)?;
    manifest.evidence_bundle.read(base)?;
    manifest.coverage.read(base)?;
    for r in &manifest.supplementary_artifacts {
        r.read(base)?;
    }
    let ids: BTreeSet<_> = manifest
        .external_sources
        .iter()
        .map(|s| s.id.clone())
        .collect();
    ensure!(
        ids.len() == manifest.external_sources.len()
            && !ids.is_empty()
            && manifest
                .issuer_semantics
                .evidence_ids
                .iter()
                .all(|id| ids.contains(id)),
        "unbound issuer semantics"
    );
    for source in &manifest.external_sources {
        ensure!(
            source.kind != EvidenceKind::LocalExecution,
            "research cannot fabricate execution"
        );
        source.artifact.read(base)?;
    }
    let mut accounts = BTreeMap::new();
    let mut txs: BTreeMap<String, (Value, RawEvidence)> = BTreeMap::new();
    let mut searches = Vec::new();
    let mut errors = Vec::new();
    let mut requested = BTreeSet::new();
    let mut request_count = 0;
    let mut archives = BTreeSet::new();
    for artifact in &manifest.archives {
        ensure!(
            archives.insert(&artifact.sha256),
            "duplicate research archive"
        );
        let archive: Value = serde_json::from_slice(&artifact.read(base)?)?;
        ensure!(
            archive["schema_version"] == 1,
            "unsupported RPC archive schema"
        );
        for (i, r) in archive["records"]
            .as_array()
            .context("archive lacks records")?
            .iter()
            .enumerate()
        {
            let prefix = format!("/records/{i}/response/result");
            let result = &r["response"]["result"];
            if !r["response"]["error"].is_null()
                || !r["transport_error"].is_null()
                || result.is_null()
            {
                errors.push(ref_at(artifact, format!("/records/{i}")));
            }
            match r["method"].as_str().context("RPC method absent")? {
                "getMultipleAccounts" if !result.is_null() => {
                    let addresses = r["params"][0]
                        .as_array()
                        .context("account addresses absent")?;
                    let values = result["value"]
                        .as_array()
                        .context("account values absent")?;
                    ensure!(
                        addresses.len() == values.len(),
                        "account batch length mismatch"
                    );
                    let slot = result["context"]["slot"]
                        .as_u64()
                        .context("account bank slot absent")?;
                    for (n, (address, raw)) in addresses.iter().zip(values).enumerate() {
                        let key = address
                            .as_str()
                            .context("account address malformed")?
                            .to_string();
                        if accounts
                            .get(&key)
                            .is_none_or(|old: &AccountRecord| old.slot <= slot)
                        {
                            accounts.insert(
                                key,
                                AccountRecord {
                                    raw: raw.clone(),
                                    slot,
                                    evidence: ref_at(artifact, format!("{prefix}/value/{n}")),
                                },
                            );
                        }
                    }
                }
                "getSignaturesForAddress" => {
                    let values = result.as_array();
                    let sigs = values
                        .into_iter()
                        .flatten()
                        .map(|s| {
                            s["signature"]
                                .as_str()
                                .context("signature absent")
                                .map(str::to_string)
                        })
                        .collect::<Result<Vec<_>>>()?;
                    searches.push(SearchQuery {
                        address: r["params"][0]
                            .as_str()
                            .context("searched address absent")?
                            .into(),
                        requested_limit: r["params"][1]["limit"]
                            .as_u64()
                            .context("signature bound absent")?,
                        returned: sigs.len(),
                        newest_slot: values
                            .into_iter()
                            .flatten()
                            .filter_map(|v| v["slot"].as_u64())
                            .max(),
                        oldest_slot: values
                            .into_iter()
                            .flatten()
                            .filter_map(|v| v["slot"].as_u64())
                            .min(),
                        signatures: sigs,
                        evidence: ref_at(artifact, prefix),
                    });
                }
                "getTransaction" => {
                    request_count += 1;
                    let sig = r["params"][0]
                        .as_str()
                        .context("transaction signature absent")?
                        .to_string();
                    requested.insert(sig.clone());
                    if !result.is_null() {
                        if let Some((old, _)) = txs.get(&sig) {
                            ensure!(old == result, "same transaction differs across archives");
                        } else {
                            txs.insert(sig, (result.clone(), ref_at(artifact, prefix)));
                        }
                    }
                }
                "getMultipleAccounts" => {}
                _ => anyhow::bail!("unexpected RPC investigation method"),
            }
        }
    }
    let retry_count = request_count - requested.len();
    ensure!(
        searches.len() <= manifest.limits.maximum_signature_queries
            && searches.iter().map(|q| q.returned).sum::<usize>()
                <= manifest.limits.maximum_returned_signatures
            && requested.len() <= manifest.limits.maximum_unique_transaction_samples
            && retry_count <= manifest.limits.maximum_transaction_retries,
        "research exceeds bounded search plan"
    );
    let searched_sigs: BTreeSet<_> = searches.iter().flat_map(|q| &q.signatures).collect();
    ensure!(
        requested.iter().all(|s| searched_sigs.contains(s)),
        "transaction outside searched signature set"
    );
    let source = mint(&manifest.source_asset, &accounts)?;
    let successor = mint(&manifest.destination_asset, &accounts)?;
    let shared = authorities(&source.configuration)
        .intersection(&authorities(&successor.configuration))
        .cloned()
        .collect();
    let mut program_ids = BTreeSet::new();
    for (tx, _) in txs.values() {
        for ix in tx["transaction"]["message"]["instructions"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(
                tx["meta"]["innerInstructions"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .flat_map(|i| i["instructions"].as_array().into_iter().flatten()),
            )
        {
            program_ids.insert(
                ix["programId"]
                    .as_str()
                    .context("transaction program absent")?
                    .to_string(),
            );
        }
    }
    let programs = program_ids
        .iter()
        .map(|p| Ok((p.clone(), program(p, &accounts)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut relationships = Vec::new();
    let mut derived = BTreeSet::new();
    // Administrative PDA relationships are protocol facts, not official conversion identity.
    let squads: Address = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf".parse()?;
    for (address, a) in &accounts {
        let mut pdas = Vec::new();
        if !a.raw.is_null() && a.raw["owner"] == squads.to_string() {
            let b = decode::raw_account_bytes(&a.raw)?;
            let discriminator = sha2::Sha256::digest(b"account:Multisig");
            if b.len() >= 95 && b[..8] == discriminator[..8] {
                let create: [u8; 32] = b[8..40].try_into()?;
                let (pda, bump) =
                    Address::find_program_address(&[b"multisig", b"multisig", &create], &squads);
                ensure!(pda.to_string() == *address, "Squads multisig PDA mismatch");
                derived.insert(address.clone());
                let threshold = u16::from_le_bytes(b[72..74].try_into()?);
                pdas.push(json!({"program":squads.to_string(),"kind":"multisig","create_key":Address::new_from_array(create).to_string(),"bump":bump,"threshold":threshold,"time_lock_seconds":u32::from_le_bytes(b[74..78].try_into()?)}));
                for index in 0..=255u8 {
                    let (vault, vb) = Address::find_program_address(
                        &[b"multisig", pda.as_ref(), b"vault", &[index]],
                        &squads,
                    );
                    if accounts.contains_key(&vault.to_string()) {
                        derived.insert(vault.to_string());
                        pdas.push(json!({"program":squads.to_string(),"kind":"vault","index":index,"address":vault.to_string(),"bump":vb}));
                    }
                }
            }
        }
        if !a.raw.is_null()
            && a.raw["owner"] == crate::lifecycle::exposure::meteora_dlmm::PROGRAM_ID
        {
            if let Ok(pool) = crate::lifecycle::exposure::meteora_dlmm::decode_pool(
                address,
                &a.raw,
                &manifest.source_asset,
            ) {
                derived.insert(address.clone());
                for v in &pool.vaults {
                    derived.insert(v.to_string());
                }
                pdas.push(json!({"kind":"observed_market_pool_not_official_mechanism","program":crate::lifecycle::exposure::meteora_dlmm::PROGRAM_ID,"mints":pool.mints.map(|m|m.to_string()),"vaults":pool.vaults.map(|v|v.to_string()),"decoded_fields":pool.decoded_fields}));
            }
        }
        relationships.push(AccountRelationship {
            address: address.clone(),
            slot: a.slot,
            runtime_owner: a.raw["owner"].as_str().map(str::to_string),
            executable: a.raw["executable"].as_bool(),
            raw_data_sha256: if a.raw.is_null() {
                None
            } else {
                Some(sha256(&decode::raw_account_bytes(&a.raw)?))
            },
            evidence: a.evidence.clone(),
            verified_pdas: pdas,
        });
    }
    let transactions = txs
        .iter()
        .map(|(s, (t, r))| {
            observation(
                s,
                t,
                r.clone(),
                &manifest.source_asset,
                &manifest.destination_asset,
                &programs,
                &derived,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let candidates=transactions.iter().filter(|t|t.touches_source || t.touches_successor || t.instruction_types.iter().any(|s|s=="withdrawWithheldTokensFromMint")).map(|t| {
        OfficialTransitionMechanism {id:format!("observed-{}",t.signature),source_asset:manifest.source_asset.clone(),destination_asset:manifest.destination_asset.clone(),mechanism_type:if t.touches_source && t.touches_successor {MechanismType::Swap}else{MechanismType::Unknown},identity:MechanismIdentity {issuer_semantics_bound:false,observed_official_transaction:false,exact_account_plan_verified:false,source_destination_pair_observed:t.touches_source && t.touches_successor},programs:t.programs.clone(),required_accounts:t.account_list.as_array().unwrap().iter().filter_map(|k|k["pubkey"].as_str().map(str::to_string)).collect(),required_signers:t.signer_list.iter().map(|s|RequiredSigner {address:s.clone(),role:AuthorityRole::Unknown,possession_known:false,assumed_locally:false}).collect(),eligibility_inputs:vec![EligibilityInput::Unknown],onchain_evidence:vec![format!("{}#{}",t.evidence.artifact.file,t.evidence.pointer)],external_policy_evidence:manifest.issuer_semantics.evidence_ids.clone(),blocking_requirements:vec![t.interpretation.clone(),"Observed instruction/account list belongs to the sampled transaction, not an executable official plan for the canonical holder.".into()]}
    }).collect::<Vec<_>>();
    let selected=OfficialTransitionMechanism {id:"official-transition-unestablished".into(),source_asset:manifest.source_asset.clone(),destination_asset:manifest.destination_asset.clone(),mechanism_type:MechanismType::Unknown,identity:MechanismIdentity {issuer_semantics_bound:false,observed_official_transaction:false,exact_account_plan_verified:false,source_destination_pair_observed:transactions.iter().any(|t|t.touches_source && t.touches_successor)},programs:vec![],required_accounts:vec![],required_signers:vec![],eligibility_inputs:vec![EligibilityInput::Unknown],onchain_evidence:manifest.archives.iter().map(|r|r.file.clone()).collect(),external_policy_evidence:manifest.issuer_semantics.evidence_ids.clone(),blocking_requirements:vec!["No independently bound official program/instruction/account plan, conversion terms or issuer completion criteria were established.".into(),"Private eligibility requirements remain unknown; administrative signing in sampled mint/fee operations must not be assumed for conversion.".into()]};
    let scope = TransitionScope {
        entity_id: entity.clone(),
        mechanism_id: selected.id.clone(),
        context_id: "unestablished-official-mechanism".into(),
        exact_input_raw: prior.observed_public_balance_raw.clone(),
        source_asset: manifest.source_asset.clone(),
        destination_asset: manifest.destination_asset.clone(),
    };
    let assessment = assess(&selected, &scope, None);
    let mut discovery: DiscoveryManifest =
        serde_json::from_slice(&manifest.prior_discovery.read(base)?)?;
    let mut new_ids = Vec::new();
    for (n, artifact) in manifest.archives.iter().enumerate() {
        let id = format!("transition-public-rpc-{n}");
        new_ids.push(id.clone());
        discovery.sources.push(DiscoverySource {id,kind:EvidenceKind::ObservedOnchainState,reference:"rpc:bounded-official-transition-investigation".into(),artifact:artifact.clone(),description:"Bounded current public mint, program, administrative and transaction observations; no official execution proof.".into()});
    }
    let d = discovery
        .paths
        .iter_mut()
        .find(|p| p.path_type == ExitPathType::OfficialTransition)
        .unwrap();
    d.evidence_ids.extend(new_ids);
    d.reason = assessment.reason.clone();
    d.facts.push(format!("Bounded investigation: {} address queries, {} returned signature entries, {} unique transactions, {} available, {} retry requests. No official executor established.",searches.len(),searches.iter().map(|q|q.returned).sum::<usize>(),requested.len(),txs.len(),retry_count));
    d.limitations.push("Observed DEX routes, source burn/closure, successor MintTo and fee administration are not independently bound official conversion evidence. Unknown eligibility is not evidence of an actual KYC/backend dependency.".into());
    discovery.validate()?;
    prior.validate(
        &base.join(&manifest.evidence_bundle.file),
        snapshot,
        scenario,
        &base.join(&manifest.coverage.file),
        &base.join(&manifest.prior_discovery.file),
    )?;
    let bundle: resolution::phase7::ResolutionBundle =
        serde_json::from_slice(&manifest.evidence_bundle.read(base)?)?;
    let bundle_base = base
        .join(&manifest.evidence_bundle.file)
        .parent()
        .unwrap()
        .to_path_buf();
    let impact: crate::lifecycle::consequence::LifecycleImpactReport =
        serde_json::from_slice(&bundle.impact.read(&bundle_base)?)?;
    let target = impact
        .entities
        .iter()
        .find(|e| e.entity_id == entity)
        .context("impact entity missing")?;
    let official = resolution::LifecyclePathResolver::resolve(target, &discovery, &[])?
        .into_iter()
        .find(|p| p.path_type == ExitPathType::OfficialTransition)
        .unwrap();
    let mut updated = prior.clone();
    updated.discovery_sha256 = digest(&discovery)?;
    *updated
        .paths
        .iter_mut()
        .find(|p| p.path_type == ExitPathType::OfficialTransition)
        .unwrap() = official;
    ensure!(
        updated
            .paths
            .iter()
            .filter(|p| p.path_type != ExitPathType::OfficialTransition)
            .eq(prior
                .paths
                .iter()
                .filter(|p| p.path_type != ExitPathType::OfficialTransition)),
        "other lifecycle paths changed"
    );
    Ok(OfficialTransitionReport {schema_version:1,entity_id:entity,retained_owner:prior.owner_authority,observed_balance_raw:prior.observed_public_balance_raw,research_manifest_sha256:sha256(&std::fs::read(manifest_path)?),issuer_semantics:manifest.issuer_semantics,external_sources:manifest.external_sources,source_verification:source,successor_verification:successor,shared_base_authorities:shared,searches,transaction_requests:request_count,transaction_retry_requests:retry_count,unique_transaction_samples:requested.len(),available_transaction_samples:txs.len(),rpc_errors:errors,transactions,programs:programs.into_values().collect(),account_relationships:relationships,candidate_mechanisms:candidates,selected_mechanism:selected,assessment,exact_tested_input_raw:None,stop_condition:manifest.stop_condition,limitations:vec!["Bounded RPC samples and account observations are not complete historical enumeration, signed inclusion proofs or a creation-slot proof.".into(),"Current account/program captures are separate banks from sampled historical transactions; they are not transaction pre-state or an official execution fixture.".into(),"Issuer webpages and metadata are external assertions; asset bytes alone establish no legal issuer relationship or holder entitlement.".into(),"No official transition VM execution, invented ratio, issuer signature, KYC state, backend payload or successor credit was supplied.".into(),"No proof inherits from prior transfer/USDC execution or the unrelated sampled holders/market contexts.".into()],updated_discovery:discovery,updated_resolution:updated})
}
use sha2::Digest;
