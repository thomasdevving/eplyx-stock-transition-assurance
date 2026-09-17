//! One bounded adapter: Meteora DLMM PermissionlessV2, 904-byte LbPair, state version 1.
//! Layout and PDA rules are pinned to the official SDK revision below.
use anyhow::{ensure, Context, Result};
use borsh::BorshDeserialize;
use serde_json::{json, Value};
use solana_address::Address;

use super::{
    AdapterIdentity, AdapterRun, ExposureAdapter, LiquidityAsset, LiquidityExposure,
    ProtocolEvidence, SnapshotVaultLink,
};
use crate::lifecycle::{
    decode::{self, LEGACY_PROGRAM, TOKEN_2022_PROGRAM},
    LifecycleSnapshot,
};

pub const PROGRAM_ID: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";
pub const SDK_REVISION: &str = "576919e3e4368e542c402f000b4264724f7f23ec";
pub const ADAPTER_VERSION: &str = "1";
pub const DISCRIMINATOR: [u8; 8] = [33, 11, 49, 98, 181, 101, 177, 13];
pub const ACCOUNT_LEN: usize = 904;

// The IDL zero-copy layout includes explicit padding. Opaque regions are fields
// irrelevant to custody discovery (fees/rewards/bins); complete bytes are retained.
#[derive(BorshDeserialize)]
struct LbPairLayout {
    _parameters: [u8; 32],
    _v_parameters: [u8; 32],
    bump_seed: [u8; 1],
    bin_step_seed: [u8; 2],
    pair_type: u8,
    active_id: i32,
    bin_step: u16,
    status: u8,
    require_base_factor_seed: u8,
    _base_factor_seed: [u8; 2],
    activation_type: u8,
    creator_pool_on_off_control: u8,
    token_x_mint: [u8; 32],
    token_y_mint: [u8; 32],
    reserve_x: [u8; 32],
    reserve_y: [u8; 32],
    _protocol_fee: [u8; 16],
    _padding_1: [u8; 32],
    _reward_infos: [u8; 288],
    _oracle: [u8; 32],
    _bin_array_bitmap: [u8; 128],
    _last_updated_at: i64,
    _padding_2: [u8; 32],
    _pre_activation_swap_address: [u8; 32],
    base_key: [u8; 32],
    _activation_point: u64,
    _pre_activation_duration: u64,
    _padding_3: [u8; 8],
    _padding_4: u64,
    _creator: [u8; 32],
    token_mint_x_program_flag: u8,
    token_mint_y_program_flag: u8,
    version: u8,
    _reserved: [u8; 21],
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedPool {
    pub pool: Address,
    pub mints: [Address; 2],
    pub vaults: [Address; 2],
    pub token_programs: [String; 2],
    pub decoded_fields: Value,
}

pub struct MeteoraDlmmAdapter;

pub fn decode_pool(pool: &str, raw: &Value, target_mint: &str) -> Result<DecodedPool> {
    let pool: Address = pool.parse().context("invalid DLMM pool address")?;
    let bytes = decode::account_bytes(raw, PROGRAM_ID)
        .context("candidate is not a non-executable DLMM-owned pool")?;
    ensure!(
        bytes.len() == ACCOUNT_LEN,
        "unsupported DLMM account length (expected 904)"
    );
    ensure!(
        bytes[..8] == DISCRIMINATOR,
        "candidate lacks DLMM LbPair discriminator"
    );
    let layout =
        LbPairLayout::try_from_slice(&bytes[8..]).context("malformed DLMM LbPair layout")?;
    ensure!(
        layout.version == 1 && layout.pair_type == 3,
        "unsupported DLMM state version / pair type (only version 1 PermissionlessV2)"
    );
    ensure!(
        layout.status <= 1
            && layout.activation_type <= 1
            && layout.require_base_factor_seed <= 1
            && layout.creator_pool_on_off_control <= 1,
        "invalid DLMM enum/boolean field"
    );
    ensure!(
        layout.bin_step > 0 && layout.bin_step_seed == layout.bin_step.to_le_bytes(),
        "invalid DLMM bin-step seed"
    );
    let mints = [
        Address::new_from_array(layout.token_x_mint),
        Address::new_from_array(layout.token_y_mint),
    ];
    let vaults = [
        Address::new_from_array(layout.reserve_x),
        Address::new_from_array(layout.reserve_y),
    ];
    ensure!(
        mints[0] != mints[1] && vaults[0] != vaults[1],
        "duplicate DLMM assets/vaults"
    );
    ensure!(
        mints.iter().any(|m| m.to_string() == target_mint),
        "candidate pool does not contain lifecycle mint"
    );
    let program: Address = PROGRAM_ID.parse()?;
    let (min, max) = if layout.token_x_mint < layout.token_y_mint {
        (&layout.token_x_mint, &layout.token_y_mint)
    } else {
        (&layout.token_y_mint, &layout.token_x_mint)
    };
    // Official deriveLbPairWithPresetParamWithIndexKey: base_key stores the
    // preset parameter key for this pool type. Verify both address and bump.
    let (derived_pool, bump) =
        Address::find_program_address(&[&layout.base_key, min, max], &program);
    ensure!(
        derived_pool == pool && layout.bump_seed[0] == bump,
        "DLMM pool PDA / bump mismatch"
    );
    for i in 0..2 {
        let (reserve, _) =
            Address::find_program_address(&[pool.as_ref(), mints[i].as_ref()], &program);
        ensure!(
            reserve == vaults[i],
            "DLMM reserve is not canonical pool/mint PDA"
        );
    }
    let token_programs = [
        layout.token_mint_x_program_flag,
        layout.token_mint_y_program_flag,
    ]
    .map(|flag| match flag {
        0 => Ok(LEGACY_PROGRAM.to_string()),
        1 => Ok(TOKEN_2022_PROGRAM.to_string()),
        _ => Err(anyhow::anyhow!("unsupported DLMM token-program flag")),
    });
    let [x, y] = token_programs;
    Ok(DecodedPool {
        pool,
        mints,
        vaults,
        token_programs: [x?, y?],
        decoded_fields: json!({
        "account_type":"LbPair", "state_version":layout.version,"pair_type":"PermissionlessV2",
        "pool_bump":bump,"base_key":Address::new_from_array(layout.base_key).to_string(),
        "bin_step":layout.bin_step,"active_bin_id":layout.active_id,
        "status":if layout.status==0 {"Enabled"} else {"Disabled"},
        "token_program_flags":[layout.token_mint_x_program_flag,layout.token_mint_y_program_flag]}),
    })
}

impl ExposureAdapter for MeteoraDlmmAdapter {
    fn identity(&self) -> AdapterIdentity {
        AdapterIdentity {
            name: "meteora-dlmm".into(),
            version: ADAPTER_VERSION.into(),
            decoder_revision: SDK_REVISION.into(),
            program_id: PROGRAM_ID.into(),
        }
    }
    fn required_accounts(&self, pool: &str, raw: &Value, target_mint: &str) -> Result<Vec<String>> {
        let p = decode_pool(pool, raw, target_mint)?;
        Ok(vec![
            pool.into(),
            p.vaults[0].to_string(),
            p.vaults[1].to_string(),
            p.mints[0].to_string(),
            p.mints[1].to_string(),
            PROGRAM_ID.into(),
        ])
    }
    fn verify(
        &self,
        snapshot: &LifecycleSnapshot,
        run: &AdapterRun,
        run_index: usize,
    ) -> Result<LiquidityExposure> {
        let values = run.evidence[2].result["value"]
            .as_array()
            .context("missing DLMM verification accounts")?;
        ensure!(
            values.len() == 6 && values.iter().all(|v| !v.is_null()),
            "DLMM verification account absent"
        );
        let p = decode_pool(&run.candidate_pool, &values[0], &snapshot.asset.mint)?;
        ensure!(
            values[5]["executable"].as_bool() == Some(true),
            "DLMM program account is not executable"
        );
        ensure!(
            matches!(
                values[5]["owner"].as_str(),
                Some(
                    "BPFLoaderUpgradeab1e11111111111111111111111"
                        | "BPFLoader2111111111111111111111111111111111"
                        | "LoaderV411111111111111111111111111111111111"
                )
            ),
            "DLMM executable has unsupported runtime loader"
        );
        let decoder = format!("meteora-dlmm/{ADAPTER_VERSION}:LbPair@{SDK_REVISION}");
        let pool_evidence = ProtocolEvidence::adapter(run, run_index, 0, &decoder)?;
        let program_evidence =
            ProtocolEvidence::adapter(run, run_index, 5, "Solana runtime executable account")?;
        let mut assets = Vec::new();
        for i in 0..2 {
            let mint_config = decode::decode_mint(&values[3 + i])?;
            ensure!(
                mint_config.token_program == p.token_programs[i],
                "DLMM mint owner disagrees with pool token-program flag"
            );
            let state = decode::decode_token_account(
                &values[1 + i],
                &p.token_programs[i],
                &p.mints[i].to_string(),
                mint_config.decimals,
            )?;
            ensure!(
                state.is_initialized && state.owner == run.candidate_pool,
                "DLMM vault is uninitialized or SPL authority is not the pool PDA"
            );
            let mint_evidence = ProtocolEvidence::adapter(
                run,
                run_index,
                3 + i,
                "SPL mint / Token-2022 extensions 3.1.1",
            )?;
            let vault_evidence = ProtocolEvidence::adapter(
                run,
                run_index,
                1 + i,
                "SPL token account / Token-2022 extensions 3.1.1",
            )?;
            let phase2_link = if p.mints[i].to_string() == snapshot.asset.mint {
                let entity = snapshot.entities.iter().find(|e| e.token_account == p.vaults[i].to_string()).context("verified target vault is absent from Phase 2 snapshot; recapture production state instead of inventing a link")?;
                ensure!(
                    entity.state.owner == run.candidate_pool
                        && entity.state.mint == snapshot.asset.mint,
                    "target vault ownership/mint changed relative to Phase 2 snapshot"
                );
                let historic_pool = snapshot.evidence[entity.authority_evidence.rpc_id]
                    .result
                    .pointer(&entity.authority_evidence.pointer)
                    .context("Phase 2 pool authority evidence missing")?;
                let historic =
                    decode_pool(&run.candidate_pool, historic_pool, &snapshot.asset.mint)
                        .context("Phase 2 authority bytes do not prove the same DLMM role")?;
                ensure!(
                    historic.mints == p.mints
                        && historic.vaults == p.vaults
                        && historic.token_programs == p.token_programs,
                    "DLMM pool relationship differs from Phase 2 evidence"
                );
                ensure!(
                    snapshot.mint_config.token_program == p.token_programs[i],
                    "target mint program changed since Phase 2"
                );
                Some(SnapshotVaultLink {
                    phase2_entity_id: entity.id.clone(),
                    original_classification: entity.entity_type.clone(),
                    phase2_raw_balance: entity.state.raw_balance.clone(),
                    current_raw_balance: state.raw_balance.clone(),
                    phase2_vault_evidence: ProtocolEvidence::lifecycle(
                        snapshot,
                        &entity.token_account,
                        &entity.token_account_evidence,
                        "SPL token account / Token-2022 extensions 3.1.1",
                    )?,
                    phase2_pool_evidence: ProtocolEvidence::lifecycle(
                        snapshot,
                        &run.candidate_pool,
                        &entity.authority_evidence,
                        &decoder,
                    )?,
                })
            } else {
                None
            };
            assets.push(LiquidityAsset {
                mint: p.mints[i].to_string(),
                vault: p.vaults[i].to_string(),
                mint_config,
                state,
                mint_evidence,
                vault_evidence,
                phase2_link,
            });
        }
        Ok(LiquidityExposure { id:format!("meteora-dlmm:{}",run.candidate_pool), protocol:"Meteora".into(),
            product:"DLMM".into(),program_id:PROGRAM_ID.into(),pool_address:run.candidate_pool.clone(),
            authority:run.candidate_pool.clone(),authority_rule:"DLMM pool PDA; each reserve PDA derives from [pool, mint] and its SPL owner equals the pool".into(),
            assets, position_model:None, decoded_pool:p.decoded_fields,
            discovered_at_slot:super::context_slot(&run.evidence[2].result)?, adapter:run.adapter.clone(),
            pool_evidence,program_evidence })
    }
}
