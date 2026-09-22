//! The registered candidate mechanism: Eplyx Demo Candidate Conversion.
//!
//! It is a candidate an operator intends to deploy, built from this repository and
//! pinned by digest. It is not deployed on any cluster, carries no issuer authority
//! and is never presented as a PreStocks or SPACEX mechanism. The bank it runs
//! against is freshly captured current production state plus an explicitly proposed
//! rollout overlay; every account keeps its origin.
use super::{
    expected_output, AccountOrigin, ConversionExpectation, ConversionPlan, FixtureAccount,
};
use crate::{
    executor::{LoadedProgram, ProbeTransactionExecution},
    lifecycle::{
        decode::{self, MintConfig, TokenAccountState, LEGACY_PROGRAM, TOKEN_2022_PROGRAM},
        exposure::sha256,
    },
    probe::{
        meteora_dlmm::{self as shared, ATA_PROGRAM, CLOCK, UPGRADEABLE_LOADER},
        AccountDataDelta, ExecutionAccountEvidence, ProbeExecutionPlan, ProbePrecondition,
        TokenAccountDelta,
    },
    types::{AccountSnapshot, NamedAccount},
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_address::Address;
use solana_clock::Clock;
use solana_instruction::{account_meta::AccountMeta, Instruction};
use solana_message::Message;
use spl_token_2022_interface::{
    extension::{
        account_len, transfer_fee::TransferFeeAmount, BaseStateWithExtensions,
        BaseStateWithExtensionsMut, StateWithExtensions, StateWithExtensionsMut,
    },
    state::{Account, AccountState, Mint},
};
use std::collections::BTreeMap;

/// Candidate program identity. Derived from a published preimage, never deployed.
pub const PROGRAM_ID: &str = "He4VZWmVgtbXVmHJ3tRmbLKuNDo9WG3tw5Gr36KupJUf";
pub const PROGRAM_PREIMAGE: &str = "sha256(\"eplyx-demo-candidate-conversion-v1\")";
pub const ARTIFACT: &str = "artifacts/eplyx_demo_conversion.so";
pub const LOADER: &str = "BPFLoader2111111111111111111111111111111111";
pub const REVISION: &str = "eplyx-demo-candidate-conversion-v1";
pub const CONFIG_LEN: usize = 151;
pub const CONVERT_TAG: u8 = 1;
pub const VAULT_SEED: &[u8] = b"eplyx-candidate-vault";
pub const CONFIG_SEED: &[u8] = b"eplyx-candidate-config";
pub const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
/// Proposed lamports for synthesized overlay accounts. Proposed, not observed.
const PROPOSED_LAMPORTS: u64 = 10_000_000;

pub fn program_bytes() -> Result<Vec<u8>> {
    let path = crate::repo_root().join(ARTIFACT);
    std::fs::read(&path).with_context(|| {
        format!(
            "could not read the registered candidate mechanism {}\n\
             run `./scripts/build-programs.sh` to build it",
            path.display()
        )
    })
}

/// Check the actual VM loader input, after fixture construction and before execution.
/// The registry supplies the ABI and address, never substitute executable bytes.
pub fn assert_candidate_program_identity(
    programs: &[LoadedProgram],
    expected_bytes: &[u8],
    expected_sha256: &str,
) -> Result<()> {
    ensure!(
        sha256(expected_bytes) == expected_sha256,
        "candidate program digest mismatch"
    );
    let candidate: Vec<_> = programs
        .iter()
        .filter(|p| p.program_id.to_string() == PROGRAM_ID)
        .collect();
    ensure!(
        candidate.len() == 1,
        "candidate VM program identity mismatch"
    );
    ensure!(
        candidate[0].loader.to_string() == LOADER
            && candidate[0].bytes == expected_bytes
            && sha256(&candidate[0].bytes) == expected_sha256,
        "candidate VM program bytes differ from the validated package"
    );
    Ok(())
}

/// Canonical ProgramData addresses of the captured program headers, in order.
pub fn programdata_addresses(headers: &[Value]) -> Result<Vec<String>> {
    Ok(headers
        .iter()
        .map(shared::programdata_address)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}

/// Deterministic proposed rollout addresses. Nothing here is fetched or supplied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateOverlay {
    pub program: String,
    pub config: String,
    pub config_bump: u8,
    pub vault_authority: String,
    pub vault_bump: u8,
    pub reserve_vault: String,
    pub destination: String,
}
fn decode_hex32(digest: &str) -> Result<[u8; 32]> {
    ensure!(digest.len() == 64, "expected a sha-256 digest");
    let mut bytes = [0u8; 32];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&digest[i * 2..i * 2 + 2], 16).context("invalid digest")?;
    }
    Ok(bytes)
}
pub fn associated_token_address(
    owner: &Address,
    token_program: &Address,
    mint: &Address,
) -> Result<Address> {
    Ok(Address::find_program_address(
        &[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ATA_PROGRAM.parse()?,
    )
    .0)
}
/// Derive the whole proposed overlay from the plan digest and observed identities.
pub fn derive(
    plan_sha256: &str,
    owner: &str,
    replacement_mint: &str,
    replacement_program: &str,
) -> Result<CandidateOverlay> {
    let program: Address = PROGRAM_ID.parse()?;
    let (config, config_bump) =
        Address::find_program_address(&[CONFIG_SEED, &decode_hex32(plan_sha256)?], &program);
    let (vault_authority, vault_bump) =
        Address::find_program_address(&[VAULT_SEED, config.as_ref()], &program);
    let mint: Address = replacement_mint.parse()?;
    let token_program: Address = replacement_program.parse()?;
    Ok(CandidateOverlay {
        program: PROGRAM_ID.into(),
        config: config.to_string(),
        config_bump,
        reserve_vault: associated_token_address(&vault_authority, &token_program, &mint)?
            .to_string(),
        vault_authority: vault_authority.to_string(),
        vault_bump,
        destination: associated_token_address(&owner.parse()?, &token_program, &mint)?.to_string(),
    })
}
/// Exact bytes of the proposed candidate configuration account.
pub fn config_data(
    plan: &ConversionPlan,
    overlay: &CandidateOverlay,
    source_decimals: u8,
    replacement_decimals: u8,
) -> Result<Vec<u8>> {
    let mut data = vec![0u8; CONFIG_LEN];
    data[0] = 1;
    let key = |address: &str| -> Result<[u8; 32]> { Ok(address.parse::<Address>()?.to_bytes()) };
    data[1..33].copy_from_slice(&key(&plan.source_mint)?);
    data[33..65].copy_from_slice(&key(&plan.replacement_mint)?);
    data[65..97].copy_from_slice(&key(&overlay.reserve_vault)?);
    data[97..129].copy_from_slice(&key(&overlay.vault_authority)?);
    data[129..137].copy_from_slice(&plan.terms.ratio_numerator.to_le_bytes());
    data[137..145].copy_from_slice(&plan.terms.ratio_denominator.to_le_bytes());
    data[145] = plan.terms.rounding.code();
    data[146..148].copy_from_slice(&plan.terms.conversion_fee_bps.to_le_bytes());
    data[148] = source_decimals;
    data[149] = replacement_decimals;
    data[150] = overlay.vault_bump;
    Ok(data)
}
/// Build a proposed token account for the given observed mint, with the account
/// extensions that mint requires on initialization.
pub fn proposed_token_account(
    mint_bytes: &[u8],
    mint: &Address,
    owner: &Address,
    amount: u64,
) -> Result<Vec<u8>> {
    let length = account_len::try_calculate_account_len_from_mint_data(mint_bytes, &[])
        .map_err(|e| anyhow::anyhow!("cannot size proposed token account: {e:?}"))?;
    let mut data = vec![0u8; length];
    {
        let mint_state = StateWithExtensions::<Mint>::unpack(mint_bytes)?;
        let mut state = StateWithExtensionsMut::<Account>::unpack_uninitialized(&mut data)?;
        account_len::try_for_each_required_init_account_extension(
            mint_state.get_tlv_data(),
            |extension| state.init_account_extension_from_type(extension),
        )
        .map_err(|e| anyhow::anyhow!("cannot initialize proposed account extensions: {e:?}"))?;
        state.base = Account {
            mint: *mint,
            owner: *owner,
            amount,
            state: AccountState::Initialized,
            ..Account::default()
        };
        state.pack_base();
        state.init_account_type()?;
    }
    Ok(data)
}
fn proposed(address: &str, owner: &str, data: Vec<u8>) -> NamedAccount {
    NamedAccount {
        label: format!("proposed:{address}"),
        address: address.into(),
        account: AccountSnapshot {
            lamports: PROPOSED_LAMPORTS,
            owner: owner.into(),
            data,
            executable: false,
            rent_epoch: 0,
        },
    }
}

/// Everything the adapter needs about the current run, derived from the capture.
pub struct ConversionContext {
    pub genesis_hash: String,
    pub minimum_slot: u64,
    pub owner: String,
    pub source_program: String,
    pub source_decimals: u8,
    pub amount: u64,
}
/// The deterministic account plan of the final captured batch.
pub fn address_plan(
    plan: &ConversionPlan,
    overlay: &CandidateOverlay,
    owner: &str,
    source_program: &str,
    replacement_program: &str,
    programdata: &[String],
) -> Vec<String> {
    let mut addresses: std::collections::BTreeSet<String> = [
        plan.source_account.clone(),
        plan.source_mint.clone(),
        plan.replacement_mint.clone(),
        owner.to_string(),
        CLOCK.into(),
        ATA_PROGRAM.into(),
        source_program.to_string(),
        replacement_program.to_string(),
        overlay.destination.clone(),
    ]
    .into();
    addresses.extend(programdata.iter().cloned());
    addresses.into_iter().collect()
}

/// Reasons the registered candidate mechanism cannot be executed against this
/// configuration. These are executor/evidence boundaries, never impossibility.
pub fn unsupported(
    source_mint: &MintConfig,
    source: &TokenAccountState,
    replacement_mint: &MintConfig,
    destination: Option<&TokenAccountState>,
) -> Option<String> {
    if !source.is_initialized || source.is_frozen {
        return Some("Uninitialized or frozen source token account".into());
    }
    if !replacement_mint.is_initialized {
        return Some("The replacement address is not an initialized token mint".into());
    }
    for extension in &source_mint.extensions {
        if (extension.extension_type == "Pausable" && extension.config["paused"] == true)
            || extension.extension_type == "ConfidentialMintBurn"
        {
            return Some(format!(
                "Unsupported source configuration for candidate burn: {}",
                extension.extension_type
            ));
        }
    }
    for extension in &replacement_mint.extensions {
        if (extension.extension_type == "Pausable" && extension.config["paused"] == true)
            || (extension.extension_type == "TransferHook"
                && !extension.config["programId"].is_null())
            || extension.extension_type == "NonTransferable"
        {
            return Some(format!(
                "Unsupported replacement configuration for candidate release: {}",
                extension.extension_type
            ));
        }
    }
    if source
        .extensions
        .iter()
        .any(|e| e.extension_type.contains("Confidential"))
    {
        return Some("Confidential source accounts are unsupported".into());
    }
    if let Some(d) = destination {
        if !d.is_initialized || d.is_frozen {
            return Some(
                "The holder's replacement token account is uninitialized or frozen".into(),
            );
        }
        if d.extensions.iter().any(|e| {
            e.extension_type == "MemoTransfer" && e.config["requireIncomingTransferMemos"] == true
        }) {
            return Some("The replacement account requires an unsupported incoming memo".into());
        }
        if d.extensions
            .iter()
            .any(|e| e.extension_type.contains("Confidential"))
        {
            return Some("Confidential replacement accounts are unsupported".into());
        }
    }
    None
}

/// What the adapter proved about the bank it built, before execution.
pub struct BuiltConversion {
    pub plan: ProbeExecutionPlan,
    pub overlay: CandidateOverlay,
    pub accounts: Vec<FixtureAccount>,
    pub expectation: ConversionExpectation,
    pub source_decimals: u8,
    pub replacement_decimals: u8,
    pub replacement_program: String,
    pub source_program: String,
    pub owner: String,
    pub source_before_raw: String,
    pub destination_existed: bool,
    pub candidate_program_sha256: String,
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    plan: &ConversionPlan,
    plan_sha256: &str,
    context: &ConversionContext,
    evidence: &[crate::lifecycle::RpcEvidence],
    program: &[u8],
) -> Result<BuiltConversion> {
    ensure!(evidence.len() == 5, "incomplete conversion RPC transcript");
    for (i, record) in evidence.iter().enumerate() {
        ensure!(record.id == i, "noncanonical conversion evidence ids");
    }
    let config = |slot: u64| -> Value {
        json!({"encoding":"base64","commitment":"finalized","minContextSlot":slot})
    };
    ensure!(
        evidence[0].method == "getGenesisHash"
            && evidence[0].params == json!([])
            && evidence[0].result.as_str() == Some(&context.genesis_hash),
        "conversion capture on the wrong chain"
    );
    ensure!(
        evidence[1].method == "getAccountInfo"
            && evidence[1].params == json!([plan.source_mint, config(context.minimum_slot)]),
        "invalid source mint request"
    );
    let source_mint_slot = shared::slot(&evidence[1].result)?;
    ensure!(
        source_mint_slot >= context.minimum_slot,
        "source mint capture predates wallet discovery"
    );
    ensure!(
        evidence[2].method == "getAccountInfo"
            && evidence[2].params == json!([plan.replacement_mint, config(source_mint_slot)]),
        "invalid independent replacement mint request"
    );
    let replacement_slot = shared::slot(&evidence[2].result)?;
    ensure!(
        replacement_slot >= source_mint_slot,
        "old replacement mint context"
    );
    let replacement_raw = &evidence[2].result["value"];
    ensure!(
        !replacement_raw.is_null(),
        "the replacement mint account does not exist"
    );
    let replacement_program = replacement_raw["owner"]
        .as_str()
        .context("missing replacement runtime owner")?
        .to_string();
    ensure!(
        [LEGACY_PROGRAM, TOKEN_2022_PROGRAM].contains(&replacement_program.as_str()),
        "the replacement mint is not owned by a supported token program"
    );
    let mut programs: Vec<String> = vec![
        context.source_program.clone(),
        replacement_program.clone(),
        ATA_PROGRAM.into(),
    ];
    programs.sort();
    programs.dedup();
    ensure!(
        evidence[3].method == "getMultipleAccounts"
            && evidence[3].params == json!([programs, config(replacement_slot)]),
        "invalid conversion program-header request"
    );
    let headers = evidence[3].result["value"]
        .as_array()
        .context("missing program headers")?;
    ensure!(
        headers.len() == programs.len(),
        "incomplete program headers"
    );
    let header_slot = shared::slot(&evidence[3].result)?;
    ensure!(
        header_slot >= replacement_slot,
        "old program header context"
    );
    let programdata = programdata_addresses(headers)?;
    let overlay = derive(
        plan_sha256,
        &context.owner,
        &plan.replacement_mint,
        &replacement_program,
    )?;
    let addresses = address_plan(
        plan,
        &overlay,
        &context.owner,
        &context.source_program,
        &replacement_program,
        &programdata,
    );
    ensure!(
        evidence[4].method == "getMultipleAccounts"
            && evidence[4].params == json!([addresses, config(header_slot)]),
        "invalid final conversion account batch"
    );
    let values = evidence[4].result["value"]
        .as_array()
        .context("missing final conversion accounts")?;
    ensure!(
        values.len() == addresses.len(),
        "missing final conversion accounts"
    );
    let final_slot = shared::slot(&evidence[4].result)?;
    ensure!(final_slot >= header_slot, "old final conversion context");
    let raw: BTreeMap<&str, &Value> = addresses
        .iter()
        .map(|a| a.as_str())
        .zip(values.iter())
        .collect();
    let get = |address: &str| -> Result<&Value> {
        let value = *raw
            .get(address)
            .with_context(|| format!("missing conversion account {address}"))?;
        ensure!(!value.is_null(), "missing conversion account {address}");
        Ok(value)
    };
    let clock: Clock = shared::clock(get(CLOCK)?)?;
    ensure!(clock.slot == final_slot, "Clock does not match final bank");

    // Observed identities, re-derived from the final authoritative batch.
    let source_mint = decode::decode_mint(get(&plan.source_mint)?)?;
    let replacement_mint = decode::decode_mint(get(&plan.replacement_mint)?)?;
    ensure!(
        source_mint.token_program == context.source_program
            && source_mint.decimals == context.source_decimals,
        "the source mint program or decimal basis changed"
    );
    ensure!(
        replacement_mint.token_program == replacement_program,
        "the replacement mint program changed during capture"
    );
    let source = decode::decode_token_account(
        get(&plan.source_account)?,
        &context.source_program,
        &plan.source_mint,
        source_mint.decimals,
    )?;
    ensure!(
        source.owner == context.owner && source.mint == plan.source_mint,
        "the selected source account authority or mint changed"
    );
    let destination_raw = *raw
        .get(overlay.destination.as_str())
        .context("missing destination record")?;
    let destination = if destination_raw.is_null() {
        None
    } else {
        Some(decode::decode_token_account(
            destination_raw,
            &replacement_program,
            &plan.replacement_mint,
            replacement_mint.decimals,
        )?)
    };
    if let Some(reason) = unsupported(
        &source_mint,
        &source,
        &replacement_mint,
        destination.as_ref(),
    ) {
        anyhow::bail!("Unsupported: {reason}");
    }
    let authority_raw = get(&context.owner)?;
    let authority: Address = context.owner.parse()?;
    ensure!(
        authority.is_on_curve()
            && authority_raw["owner"] == SYSTEM_PROGRAM
            && authority_raw["executable"] == false
            && decode::raw_account_bytes(authority_raw)?.is_empty(),
        "captured holder authority is not a supported directly signing wallet"
    );
    ensure!(
        source.raw_balance.parse::<u64>()? >= context.amount && context.amount > 0,
        "insufficient source token balance for this exact candidate amount"
    );
    let expectation = expected_output(context.amount, &plan.terms)?;
    let funded: u64 = plan.reserve.funded_replacement_raw.parse()?;

    // Deployed executables, exactly as captured.
    let mut loaded = Vec::new();
    for (index, id) in programs.iter().enumerate() {
        let header = get(id)?;
        ensure!(
            shared::programdata_address(header)? == shared::programdata_address(&headers[index])?,
            "program loader link changed during capture"
        );
        let bytes = if let Some(pd) = shared::programdata_address(header)? {
            ensure!(
                Address::find_program_address(
                    &[id.parse::<Address>()?.as_ref()],
                    &UPGRADEABLE_LOADER.parse()?
                )
                .0
                .to_string()
                    == pd,
                "noncanonical ProgramData"
            );
            let state = get(&pd)?;
            ensure!(
                state["owner"] == UPGRADEABLE_LOADER && state["executable"] == false,
                "invalid ProgramData owner/state"
            );
            let b = decode::raw_account_bytes(state)?;
            ensure!(
                b.len() > 45
                    && b[..4] == 3u32.to_le_bytes()
                    && b[12] <= 1
                    && u64::from_le_bytes(b[4..12].try_into()?) < clock.slot,
                "malformed/new ProgramData"
            );
            b[45..].to_vec()
        } else {
            decode::raw_account_bytes(header)?
        };
        ensure!(bytes.starts_with(b"\x7fELF"), "captured program has no ELF");
        loaded.push(LoadedProgram {
            program_id: id.parse()?,
            loader: header["owner"].as_str().context("loader absent")?.parse()?,
            bytes,
        });
    }
    ensure!(
        program.starts_with(b"\x7fELF"),
        "the registered candidate mechanism is not an SBF ELF"
    );
    let candidate_program_sha256 = sha256(program);
    loaded.push(LoadedProgram {
        program_id: PROGRAM_ID.parse()?,
        loader: LOADER.parse()?,
        bytes: program.to_vec(),
    });

    // Observed bank, then the proposed rollout overlay. Origins never mix.
    let mut accounts = Vec::new();
    let mut fixture_accounts = Vec::new();
    let mut account_evidence = Vec::new();
    for (index, address) in addresses.iter().enumerate() {
        let value = &values[index];
        account_evidence.push(ExecutionAccountEvidence {
            address: address.clone(),
            rpc_record: 4,
            pointer: format!("/value/{index}"),
            slot: final_slot,
            exists: !value.is_null(),
            runtime_owner: value["owner"].as_str().map(str::to_string),
            raw_data_sha256: if value.is_null() {
                None
            } else {
                Some(sha256(&decode::raw_account_bytes(value)?))
            },
        });
        if value.is_null() {
            continue;
        }
        let bytes = decode::raw_account_bytes(value)?;
        fixture_accounts.push(FixtureAccount {
            address: address.clone(),
            origin: AccountOrigin::Observed,
            role: role_of(address, plan, &overlay, &context.owner, &programs),
            runtime_owner: value["owner"].as_str().context("owner")?.into(),
            lamports: value["lamports"].as_u64().context("lamports")?,
            executable: value["executable"].as_bool().context("executable")?,
            data_len: bytes.len(),
            data_sha256: sha256(&bytes),
            rpc_record: Some(4),
            pointer: Some(format!("/value/{index}")),
            slot: Some(final_slot),
            derivation: None,
        });
        accounts.push(shared::captured_account(address, value)?);
    }
    let payer = shared::payer().to_string();
    let replacement_mint_bytes = decode::raw_account_bytes(get(&plan.replacement_mint)?)?;
    let overlay_accounts = vec![
        (
            overlay.config.clone(),
            PROGRAM_ID.to_string(),
            config_data(
                plan,
                &overlay,
                source_mint.decimals,
                replacement_mint.decimals,
            )?,
            "candidate-config",
            format!(
                "PDA([{:?}, plan sha-256], candidate program)",
                String::from_utf8_lossy(CONFIG_SEED)
            ),
        ),
        (
            overlay.vault_authority.clone(),
            SYSTEM_PROGRAM.to_string(),
            vec![],
            "candidate-authority",
            format!(
                "PDA([{:?}, config], candidate program)",
                String::from_utf8_lossy(VAULT_SEED)
            ),
        ),
        (
            overlay.reserve_vault.clone(),
            replacement_program.clone(),
            proposed_token_account(
                &replacement_mint_bytes,
                &plan.replacement_mint.parse()?,
                &overlay.vault_authority.parse()?,
                funded,
            )?,
            "candidate-reserve",
            "Associated token account of the candidate authority for the replacement mint".into(),
        ),
    ];
    for (address, owner, data, role, derivation) in overlay_accounts {
        ensure!(
            !addresses.contains(&address) && address != payer,
            "proposed overlay address {address} collides with captured current state"
        );
        fixture_accounts.push(FixtureAccount {
            address: address.clone(),
            origin: AccountOrigin::Proposed,
            role: role.into(),
            runtime_owner: owner.clone(),
            lamports: PROPOSED_LAMPORTS,
            executable: false,
            data_len: data.len(),
            data_sha256: sha256(&data),
            rpc_record: None,
            pointer: None,
            slot: None,
            derivation: Some(derivation),
        });
        accounts.push(proposed(&address, &owner, data));
    }
    fixture_accounts.push(FixtureAccount {
        address: PROGRAM_ID.into(),
        origin: AccountOrigin::Proposed,
        role: "candidate-conversion-program".into(),
        runtime_owner: LOADER.into(),
        lamports: PROPOSED_LAMPORTS,
        executable: true,
        data_len: program.len(),
        data_sha256: candidate_program_sha256.clone(),
        rpc_record: None,
        pointer: None,
        slot: None,
        derivation: Some(format!(
            "Registered repository artifact {ARTIFACT}; program id {PROGRAM_PREIMAGE}. Not deployed on any cluster."
        )),
    });
    super::validate_origins(&fixture_accounts)?;
    ensure!(
        !addresses.contains(&payer),
        "synthetic payer collides with captured accounts"
    );
    accounts.push(NamedAccount {
        label: "local-fee-payer".into(),
        address: payer.clone(),
        account: AccountSnapshot {
            lamports: 1_000_000_000,
            owner: SYSTEM_PROGRAM.into(),
            data: vec![],
            executable: false,
            rent_epoch: 0,
        },
    });

    let mut instructions = vec![Instruction {
        program_id: "ComputeBudget111111111111111111111111111111".parse()?,
        accounts: vec![],
        data: [vec![2], 1_400_000u32.to_le_bytes().to_vec()].concat(),
    }];
    if destination.is_none() {
        instructions.push(Instruction {
            program_id: ATA_PROGRAM.parse()?,
            accounts: vec![
                AccountMeta::new(shared::payer(), true),
                AccountMeta::new(overlay.destination.parse()?, false),
                AccountMeta::new_readonly(authority, false),
                AccountMeta::new_readonly(plan.replacement_mint.parse()?, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM.parse()?, false),
                AccountMeta::new_readonly(replacement_program.parse()?, false),
            ],
            data: vec![1],
        });
    }
    instructions.push(Instruction {
        program_id: PROGRAM_ID.parse()?,
        accounts: vec![
            AccountMeta::new_readonly(overlay.config.parse()?, false),
            AccountMeta::new(plan.source_account.parse()?, false),
            AccountMeta::new(plan.source_mint.parse()?, false),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(overlay.reserve_vault.parse()?, false),
            AccountMeta::new_readonly(plan.replacement_mint.parse()?, false),
            AccountMeta::new(overlay.destination.parse()?, false),
            AccountMeta::new_readonly(overlay.vault_authority.parse()?, false),
            AccountMeta::new_readonly(context.source_program.parse()?, false),
            AccountMeta::new_readonly(replacement_program.parse()?, false),
        ],
        data: [vec![CONVERT_TAG], context.amount.to_le_bytes().to_vec()].concat(),
    });
    let watch = vec![
        plan.source_account.clone(),
        plan.source_mint.clone(),
        overlay.reserve_vault.clone(),
        overlay.destination.clone(),
        overlay.config.clone(),
    ];
    let execution = ProbeExecutionPlan {
        accounts,
        watch,
        programs: loaded,
        message: Message::new(&instructions, Some(&shared::payer())),
        clock,
        preconditions: vec![ProbePrecondition {
            name: "Fresh current source/mint/authority state, independently inspected replacement mint, deployed token loaders and a deterministic proposed overlay".into(),
            proven: true,
            reason: "Verified raw captured bytes, canonical loader links, mint/owner/initialized/freeze/pause/hook/confidential state, sufficient balance and non-colliding derived proposed addresses".into(),
        }],
        account_evidence,
        assumptions: vec![
            "The recorded holder is assumed locally to sign; key possession and authorization remain unknown.".into(),
            "The candidate operator authority is a program-derived address of the registered candidate mechanism and is assumed locally. It is not an issuer key and cannot establish issuer identity or control.".into(),
            "The candidate program, its configuration and the funded reserve are proposed rollout state, not observed mainnet state. Only the source account, its mint, the holder authority, the replacement mint, the token programs and the Clock are captured current production state.".into(),
            "Signature and recent-blockhash verification are disabled in the existing LiteSVM 0.16 runtime. Only the fee payer is synthetic; token accounts and deployed token program bytes are captured.".into(),
            "A successful candidate conversion is evidence about this supplied plan only. It does not establish an issuer-defined official transition, an issuer relationship, entitlement or any authorization.".into(),
        ],
    };
    Ok(BuiltConversion {
        plan: execution,
        overlay,
        accounts: fixture_accounts,
        expectation,
        source_decimals: source_mint.decimals,
        replacement_decimals: replacement_mint.decimals,
        replacement_program,
        source_program: context.source_program.clone(),
        owner: context.owner.clone(),
        source_before_raw: source.raw_balance,
        destination_existed: destination.is_some(),
        candidate_program_sha256,
    })
}
fn role_of(
    address: &str,
    plan: &ConversionPlan,
    overlay: &CandidateOverlay,
    owner: &str,
    programs: &[String],
) -> String {
    if address == plan.source_account {
        "holder-source-token-account"
    } else if address == plan.source_mint {
        "source-mint"
    } else if address == plan.replacement_mint {
        "replacement-mint"
    } else if address == owner {
        "holder-authority"
    } else if address == overlay.destination {
        "holder-replacement-token-account"
    } else if address == CLOCK {
        "clock"
    } else if programs.iter().any(|p| p == address) {
        "deployed-token-or-ata-program"
    } else {
        "deployed-program-data"
    }
    .into()
}

/// Exact reconciliation of one candidate conversion. Three fee kinds stay separate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversionDeltas {
    pub source: TokenAccountDelta,
    pub source_supply_before_raw: String,
    pub source_supply_after_raw: String,
    pub source_supply_change_raw: String,
    pub reserve: TokenAccountDelta,
    pub destination: TokenAccountDelta,
    pub source_debited_raw: String,
    pub source_burned_raw: String,
    pub conversion_fee_raw: String,
    pub convertible_raw: String,
    pub replacement_released_raw: String,
    pub replacement_public_credit_raw: String,
    pub replacement_credit_decimal: String,
    pub source_token_2022_transfer_fee_raw: String,
    pub replacement_token_2022_transfer_fee_raw: String,
    pub expected: ConversionExpectation,
    pub account_data: Vec<AccountDataDelta>,
    pub program_reported: Option<String>,
    pub candidate_program_invoked: bool,
    pub burn_cpi_observed: bool,
    pub release_cpi_observed: bool,
    pub reconciled: bool,
    pub reconciliation: Vec<String>,
}
fn token_amounts(data: &[u8]) -> Result<(u64, u64)> {
    let account = StateWithExtensions::<Account>::unpack(data)?;
    let withheld = account
        .get_extension::<TransferFeeAmount>()
        .map(|f| u64::from(f.withheld_amount))
        .unwrap_or(0);
    Ok((account.base.amount, withheld))
}
fn supply(data: &[u8]) -> Result<u64> {
    Ok(StateWithExtensions::<Mint>::unpack(data)?.base.supply)
}
fn delta(
    address: &str,
    mint: &str,
    decimals: u8,
    before: &AccountSnapshot,
    after: &AccountSnapshot,
) -> Result<(TokenAccountDelta, u64, u64)> {
    let (n, f) = token_amounts(&before.data)?;
    let (m, g) = token_amounts(&after.data)?;
    let change = i128::from(m) - i128::from(n);
    Ok((
        TokenAccountDelta {
            address: address.into(),
            mint: mint.into(),
            before_raw: n.to_string(),
            after_raw: m.to_string(),
            change_raw: change.to_string(),
            change_decimal_base_units: if change < 0 {
                format!("-{}", decode::decimal_amount((-change) as u64, decimals))
            } else {
                decode::decimal_amount(change as u64, decimals)
            },
            withheld_fee_change_raw: (i128::from(g) - i128::from(f)).to_string(),
        },
        n,
        m,
    ))
}
pub fn reconcile(
    plan: &ConversionPlan,
    built: &BuiltConversion,
    amount: u64,
    execution: &ProbeTransactionExecution,
) -> Result<ConversionDeltas> {
    let p = &built.plan;
    let before: BTreeMap<&str, &AccountSnapshot> = p
        .accounts
        .iter()
        .map(|a| (a.address.as_str(), &a.account))
        .collect();
    // A destination the transaction creates has no captured pre-state; its
    // pre-state is an empty zero balance for the observed replacement mint.
    let empty = AccountSnapshot {
        lamports: 0,
        owner: built.replacement_program.clone(),
        data: proposed_token_account(
            &before
                .get(plan.replacement_mint.as_str())
                .context("missing replacement mint")?
                .data,
            &plan.replacement_mint.parse()?,
            &built.owner.parse()?,
            0,
        )?,
        executable: false,
        rent_epoch: 0,
    };
    // A destination the failed transaction never created has neither pre- nor
    // post-state: it simply does not exist, at a zero balance.
    let after = |address: &str| -> Result<&AccountSnapshot> {
        if address == built.overlay.destination {
            return Ok(execution.post_accounts.get(address).unwrap_or(&empty));
        }
        execution
            .post_accounts
            .get(address)
            .with_context(|| format!("missing watched post-state for {address}"))
    };
    let pre = |address: &str| -> Result<&AccountSnapshot> {
        if address == built.overlay.destination {
            return Ok(before.get(address).copied().unwrap_or(&empty));
        }
        before
            .get(address)
            .copied()
            .with_context(|| format!("missing pre-state for {address}"))
    };
    let mut account_data = Vec::new();
    for address in &p.watch {
        let a = pre(address)?;
        let b = after(address)?;
        account_data.push(AccountDataDelta {
            address: address.clone(),
            before_sha256: sha256(&a.data),
            after_sha256: sha256(&b.data),
            changed_ranges: shared::data_changes(&a.data, &b.data),
        });
    }
    let source_pre = *before
        .get(plan.source_account.as_str())
        .context("missing source pre-state")?;
    let (source, source_before, source_after) = delta(
        &plan.source_account,
        &plan.source_mint,
        built.source_decimals,
        source_pre,
        after(&plan.source_account)?,
    )?;
    let reserve_pre = *before
        .get(built.overlay.reserve_vault.as_str())
        .context("missing reserve pre-state")?;
    let (reserve, reserve_before, reserve_after) = delta(
        &built.overlay.reserve_vault,
        &plan.replacement_mint,
        built.replacement_decimals,
        reserve_pre,
        after(&built.overlay.reserve_vault)?,
    )?;
    let destination_pre = pre(&built.overlay.destination)?;
    let (destination, destination_before, destination_after) = delta(
        &built.overlay.destination,
        &plan.replacement_mint,
        built.replacement_decimals,
        destination_pre,
        after(&built.overlay.destination)?,
    )?;
    let source_mint_pre = *before
        .get(plan.source_mint.as_str())
        .context("missing source mint pre-state")?;
    let supply_before = supply(&source_mint_pre.data)?;
    let supply_after = supply(&after(&plan.source_mint)?.data)?;

    let debited = source_before.saturating_sub(source_after);
    let released = reserve_before.saturating_sub(reserve_after);
    let credited = destination_after.saturating_sub(destination_before);
    let expected = &built.expectation;
    // Independently calculated replacement-side Token-2022 transfer fee. It is a
    // different fee from the candidate conversion fee and is never double counted.
    let replacement_fee = shared::transfer_fee(
        &before
            .get(plan.replacement_mint.as_str())
            .context("missing replacement mint")?
            .data,
        p.clock.epoch,
        released,
    )?;
    let reported = execution
        .logs
        .iter()
        .find(|l| l.contains("EPLYX_CANDIDATE_CONVERSION"))
        .map(|l| l.trim_start_matches("Program log: ").to_string());
    let candidate_program_invoked = execution
        .logs
        .iter()
        .any(|l| l == &format!("Program {PROGRAM_ID} invoke [1]"));
    let cpi = |tag: u8, program: &str| {
        execution
            .inner_instructions
            .iter()
            .any(|i| i.program == program && i.stack_height >= 2 && i.data.first() == Some(&tag))
    };
    let burn_cpi_observed = cpi(15, &built.source_program);
    let release_cpi_observed = cpi(12, &built.replacement_program);
    let rollback = p.watch.iter().all(|a| {
        execution.post_accounts.get(a) == before.get(a.as_str()).copied()
            || (a == &built.overlay.destination && !before.contains_key(a.as_str()))
    });
    let expected_log = format!(
        "EPLYX_CANDIDATE_CONVERSION consumed={} conversion_fee={} released={} ratio={}/{} rounding={}",
        expected.consumed_raw,
        expected.conversion_fee_raw,
        expected.replacement_gross_raw,
        plan.terms.ratio_numerator,
        plan.terms.ratio_denominator,
        plan.terms.rounding.code()
    );
    let reconciled = if execution.success {
        candidate_program_invoked
            && burn_cpi_observed
            && release_cpi_observed
            && reported.as_deref() == Some(expected_log.as_str())
            && debited == amount
            && expected.consumed_raw == amount.to_string()
            && supply_before.checked_sub(supply_after) == Some(amount)
            && released.to_string() == expected.replacement_gross_raw
            && credited == released - replacement_fee
            && destination.withheld_fee_change_raw == replacement_fee.to_string()
            && source.withheld_fee_change_raw == "0"
            && reserve.withheld_fee_change_raw == "0"
            && before
                .get(built.overlay.config.as_str())
                .map(|a| &a.data)
                .map(|d| sha256(d))
                == Some(sha256(&after(&built.overlay.config)?.data))
    } else {
        debited == 0 && released == 0 && credited == 0 && rollback
    };
    Ok(ConversionDeltas {
        source,
        source_supply_before_raw: supply_before.to_string(),
        source_supply_after_raw: supply_after.to_string(),
        source_supply_change_raw: (i128::from(supply_after) - i128::from(supply_before)).to_string(),
        reserve,
        destination,
        source_debited_raw: debited.to_string(),
        source_burned_raw: supply_before.saturating_sub(supply_after).to_string(),
        conversion_fee_raw: expected.conversion_fee_raw.clone(),
        convertible_raw: expected.convertible_raw.clone(),
        replacement_released_raw: released.to_string(),
        replacement_public_credit_raw: credited.to_string(),
        replacement_credit_decimal: decode::decimal_amount(credited, built.replacement_decimals),
        source_token_2022_transfer_fee_raw: "0".into(),
        replacement_token_2022_transfer_fee_raw: replacement_fee.to_string(),
        expected: expected.clone(),
        account_data,
        program_reported: reported.clone(),
        candidate_program_invoked,
        burn_cpi_observed,
        release_cpi_observed,
        reconciled,
        reconciliation: vec![
            format!("Source consumption: the holder account debited {debited} raw and the captured source mint supply fell by {} raw. The source is burned, not transferred, so no source Token-2022 transfer fee applies.", supply_before.saturating_sub(supply_after)),
            format!("Candidate terms: conversion fee {} raw taken first, {} raw convertible, ratio {}/{} with {:?} rounding, expected release {} raw.", expected.conversion_fee_raw, expected.convertible_raw, plan.terms.ratio_numerator, plan.terms.ratio_denominator, plan.terms.rounding, expected.replacement_gross_raw),
            format!("Replacement delivery: the proposed reserve fell by {released} raw and the holder's replacement account rose by {credited} raw with {replacement_fee} raw withheld as a replacement-side Token-2022 transfer fee. Conversion fee and transfer fees are counted separately."),
            format!("The registered candidate program executed and reported its own arithmetic: {}.", reported.clone().unwrap_or_else(|| "no candidate report".into())),
            format!("Watched rollback on failed execution: {rollback}; transaction fees are paid separately by the synthetic payer."),
        ],
    })
}
