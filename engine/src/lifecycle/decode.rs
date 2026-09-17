//! Official SPL layouts and Token-2022 TLV parsing. Unknown extensions fail closed.
use anyhow::{bail, ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_program_pack::{IsInitialized, Pack};
use spl_token_2022_interface::{
    extension::{
        self as ext, AccountType, BaseState, BaseStateWithExtensions, ExtensionType,
        StateWithExtensions, StateWithExtensionsMut,
    },
    state::{Account, AccountState, Mint},
};
use spl_token_group_interface::state::{TokenGroup, TokenGroupMember};
use spl_token_metadata_interface::state::TokenMetadata;

pub const LEGACY_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenExtension {
    pub extension_type: String,
    pub type_id: u16,
    pub config: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MintConfig {
    pub token_program: String,
    pub is_token_2022: bool,
    pub decimals: u8,
    /// Base token quantities are strings to preserve u64 precision in JSON consumers.
    pub raw_supply: String,
    pub decimal_supply: String,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub is_initialized: bool,
    pub extensions: Vec<TokenExtension>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenAccountState {
    pub mint: String,
    /// SPL owner authority, distinct from the account's runtime Token program owner.
    pub owner: String,
    pub raw_balance: String,
    /// Exact decimal amount; intentionally excludes scaled/interest-bearing display transforms.
    pub ui_balance: String,
    pub ui_balance_basis: String,
    pub account_state: String,
    pub delegate: Option<String>,
    pub delegated_amount: String,
    pub close_authority: Option<String>,
    pub native_reserve: Option<String>,
    pub is_frozen: bool,
    pub is_initialized: bool,
    pub has_active_delegate: bool,
    pub extensions: Vec<TokenExtension>,
}

pub fn decimal_amount(amount: u64, decimals: u8) -> String {
    let mut s = amount.to_string();
    if decimals == 0 {
        return s;
    }
    let d = decimals as usize;
    if s.len() <= d {
        s = format!("{}{}", "0".repeat(d + 1 - s.len()), s);
    }
    s.insert(s.len() - d, '.');
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

pub fn account_bytes(raw: &Value, expected_program: &str) -> Result<Vec<u8>> {
    ensure!(
        raw["owner"].as_str() == Some(expected_program),
        "account has unexpected runtime program owner"
    );
    ensure!(
        raw["executable"].as_bool() == Some(false),
        "token account is executable or missing executable flag"
    );
    raw_account_bytes(raw)
}

pub fn raw_account_bytes(raw: &Value) -> Result<Vec<u8>> {
    ensure!(
        raw["data"][1].as_str() == Some("base64"),
        "expected complete base64 account data"
    );
    let bytes = STANDARD
        .decode(raw["data"][0].as_str().context("missing base64 data")?)
        .context("invalid base64 account data")?;
    if let Some(space) = raw.get("space") {
        ensure!(
            space.as_u64() == Some(bytes.len() as u64),
            "truncated account data / space mismatch"
        );
    }
    Ok(bytes)
}

fn check_program(program: &str) -> Result<()> {
    ensure!(
        program == LEGACY_PROGRAM || program == TOKEN_2022_PROGRAM,
        "unsupported token program {program}"
    );
    Ok(())
}

pub fn decode_mint(raw: &Value) -> Result<MintConfig> {
    let program = raw["owner"].as_str().context("mint lacks runtime owner")?;
    check_program(program)?;
    let bytes = account_bytes(raw, program)?;
    let (mint, extensions) = if program == TOKEN_2022_PROGRAM {
        let state = StateWithExtensions::<Mint>::unpack(&bytes)
            .context("malformed or uninitialized Token-2022 mint")?;
        (state.base, decode_extensions(&state, AccountType::Mint)?)
    } else {
        (
            Mint::unpack(&bytes).context("malformed or uninitialized legacy mint")?,
            vec![],
        )
    };
    Ok(MintConfig {
        token_program: program.into(),
        is_token_2022: program == TOKEN_2022_PROGRAM,
        decimals: mint.decimals,
        raw_supply: mint.supply.to_string(),
        decimal_supply: decimal_amount(mint.supply, mint.decimals),
        mint_authority: mint.mint_authority.map(|v| v.to_string()).into(),
        freeze_authority: mint.freeze_authority.map(|v| v.to_string()).into(),
        is_initialized: mint.is_initialized,
        extensions,
    })
}

pub fn decode_token_account(
    raw: &Value,
    program: &str,
    mint: &str,
    decimals: u8,
) -> Result<TokenAccountState> {
    check_program(program)?;
    let mut bytes = account_bytes(raw, program)?;
    let (base, extensions) = if program == TOKEN_2022_PROGRAM {
        ensure!(
            bytes.len() >= Account::LEN,
            "Token-2022 account shorter than base layout"
        );
        let base = Account::unpack_unchecked(&bytes[..Account::LEN])
            .context("malformed token account base")?;
        if base.is_initialized() {
            let state = StateWithExtensions::<Account>::unpack(&bytes)
                .context("malformed Token-2022 account")?;
            (state.base, decode_extensions(&state, AccountType::Account)?)
        } else {
            let state = StateWithExtensionsMut::<Account>::unpack_uninitialized(&mut bytes)
                .context("malformed uninitialized Token-2022 account")?;
            (state.base, decode_extensions(&state, AccountType::Account)?)
        }
    } else {
        (
            Account::unpack_unchecked(&bytes).context("malformed legacy token account")?,
            vec![],
        )
    };
    ensure!(
        base.mint.to_string() == mint,
        "token account belongs to a different mint"
    );
    Ok(TokenAccountState {
        mint: mint.into(),
        owner: base.owner.to_string(),
        raw_balance: base.amount.to_string(),
        ui_balance: decimal_amount(base.amount, decimals),
        ui_balance_basis: "raw_balance / 10^decimals; no UI transform".into(),
        account_state: match base.state {
            AccountState::Uninitialized => "Uninitialized",
            AccountState::Initialized => "Initialized",
            AccountState::Frozen => "Frozen",
        }
        .into(),
        delegate: base.delegate.map(|v| v.to_string()).into(),
        delegated_amount: base.delegated_amount.to_string(),
        close_authority: base.close_authority.map(|v| v.to_string()).into(),
        native_reserve: base.is_native.map(|v| v.to_string()).into(),
        is_frozen: base.is_frozen(),
        is_initialized: base.is_initialized(),
        has_active_delegate: base.delegate.is_some() && base.delegated_amount > 0,
        extensions,
    })
}

fn decode_extensions<S: BaseState + Pack>(
    state: &impl BaseStateWithExtensions<S>,
    account_type: AccountType,
) -> Result<Vec<TokenExtension>> {
    let types = state
        .get_extension_types()
        .context("unsupported or malformed Token-2022 TLV")?;
    let mut seen = std::collections::BTreeSet::new();
    let mut result = Vec::new();
    // SPL's parser accepts allocation padding. Reject nonzero bytes after a terminator,
    // duplicate TLVs, wrong base types and incorrect marker lengths explicitly.
    let tlv = state.get_tlv_data();
    let mut pos = 0;
    for t in types {
        ensure!(seen.insert(u16::from(t)), "duplicate Token-2022 extension");
        ensure!(
            t.get_account_type() == account_type,
            "extension on wrong account type"
        );
        ensure!(pos + 4 <= tlv.len(), "truncated TLV header");
        let len = u16::from_le_bytes([tlv[pos + 2], tlv[pos + 3]]) as usize;
        pos = pos.checked_add(4 + len).context("TLV length overflow")?;
        ensure!(pos <= tlv.len(), "truncated TLV payload");
        macro_rules! config {
            ($ty:ty) => {
                serde_json::to_value(
                    state
                        .get_extension::<$ty>()
                        .context(format!("malformed {t:?}"))?,
                )?
            };
        }
        macro_rules! marker {
            ($ty:ty) => {{
                let _ = state
                    .get_extension::<$ty>()
                    .context(format!("malformed {t:?}"))?;
                json!({})
            }};
        }
        let config = match t {
            ExtensionType::TransferFeeConfig => config!(ext::transfer_fee::TransferFeeConfig),
            ExtensionType::TransferFeeAmount => config!(ext::transfer_fee::TransferFeeAmount),
            ExtensionType::MintCloseAuthority => {
                config!(ext::mint_close_authority::MintCloseAuthority)
            }
            ExtensionType::ConfidentialTransferMint => {
                let v = state
                    .get_extension::<ext::confidential_transfer::ConfidentialTransferMint>()?;
                json!({"authority": Option::<solana_address::Address>::from(v.authority).map(|a|a.to_string()), "autoApproveNewAccounts":bool::from(v.auto_approve_new_accounts), "auditorElgamalPubkey":v.auditor_elgamal_pubkey})
            }
            ExtensionType::ConfidentialTransferAccount => {
                let v = state
                    .get_extension::<ext::confidential_transfer::ConfidentialTransferAccount>()?;
                json!({"approved":bool::from(v.approved), "elgamalPubkey":v.elgamal_pubkey.to_string(),
                    "pendingBalanceLo":v.pending_balance_lo.to_string(), "pendingBalanceHi":v.pending_balance_hi.to_string(),
                    "availableBalance":v.available_balance.to_string(), "decryptableAvailableBalance":v.decryptable_available_balance.to_string(),
                    "allowConfidentialCredits":bool::from(v.allow_confidential_credits), "allowNonConfidentialCredits":bool::from(v.allow_non_confidential_credits),
                    "pendingBalanceCreditCounter":u64::from(v.pending_balance_credit_counter).to_string(),
                    "maximumPendingBalanceCreditCounter":u64::from(v.maximum_pending_balance_credit_counter).to_string(),
                    "expectedPendingBalanceCreditCounter":u64::from(v.expected_pending_balance_credit_counter).to_string(),
                    "actualPendingBalanceCreditCounter":u64::from(v.actual_pending_balance_credit_counter).to_string()})
            }
            ExtensionType::DefaultAccountState => {
                let v = state.get_extension::<ext::default_account_state::DefaultAccountState>()?;
                let decoded =
                    AccountState::try_from(v.state).context("invalid default account state")?;
                json!({"state": format!("{decoded:?}"), "rawState": v.state})
            }
            ExtensionType::ImmutableOwner => marker!(ext::immutable_owner::ImmutableOwner),
            ExtensionType::MemoTransfer => config!(ext::memo_transfer::MemoTransfer),
            ExtensionType::NonTransferable => marker!(ext::non_transferable::NonTransferable),
            ExtensionType::InterestBearingConfig => {
                config!(ext::interest_bearing_mint::InterestBearingConfig)
            }
            ExtensionType::CpiGuard => config!(ext::cpi_guard::CpiGuard),
            ExtensionType::PermanentDelegate => config!(ext::permanent_delegate::PermanentDelegate),
            ExtensionType::NonTransferableAccount => {
                marker!(ext::non_transferable::NonTransferableAccount)
            }
            ExtensionType::TransferHook => config!(ext::transfer_hook::TransferHook),
            ExtensionType::TransferHookAccount => config!(ext::transfer_hook::TransferHookAccount),
            ExtensionType::ConfidentialTransferFeeConfig => {
                let v = state
                    .get_extension::<ext::confidential_transfer_fee::ConfidentialTransferFeeConfig>(
                    )?;
                json!({"authority":Option::<solana_address::Address>::from(v.authority).map(|a|a.to_string()),
                    "withdrawWithheldAuthorityElgamalPubkey":v.withdraw_withheld_authority_elgamal_pubkey.to_string(),
                    "harvestToMintEnabled":bool::from(v.harvest_to_mint_enabled), "withheldAmount":v.withheld_amount.to_string()})
            }
            ExtensionType::ConfidentialTransferFeeAmount => {
                let v = state
                    .get_extension::<ext::confidential_transfer_fee::ConfidentialTransferFeeAmount>(
                    )?;
                json!({"withheldAmount":v.withheld_amount.to_string()})
            }
            ExtensionType::MetadataPointer => config!(ext::metadata_pointer::MetadataPointer),
            ExtensionType::TokenMetadata => {
                let v = state
                    .get_variable_len_extension::<TokenMetadata>()
                    .context("malformed token metadata")?;
                json!({"updateAuthority":Option::<solana_address::Address>::from(v.update_authority).map(|a|a.to_string()),
                    "mint":v.mint.to_string(), "name":v.name, "symbol":v.symbol, "uri":v.uri, "additionalMetadata":v.additional_metadata})
            }
            ExtensionType::GroupPointer => config!(ext::group_pointer::GroupPointer),
            ExtensionType::GroupMemberPointer => {
                config!(ext::group_member_pointer::GroupMemberPointer)
            }
            ExtensionType::TokenGroup => {
                let v = state.get_extension::<TokenGroup>()?;
                json!({"updateAuthority": Option::<solana_address::Address>::from(v.update_authority).map(|a| a.to_string()), "mint": v.mint.to_string(), "size": u64::from(v.size).to_string(), "maxSize": u64::from(v.max_size).to_string()})
            }
            ExtensionType::TokenGroupMember => {
                let v = state.get_extension::<TokenGroupMember>()?;
                json!({"mint": v.mint.to_string(), "group": v.group.to_string(), "memberNumber": u64::from(v.member_number).to_string()})
            }
            ExtensionType::ConfidentialMintBurn => {
                let v =
                    state.get_extension::<ext::confidential_mint_burn::ConfidentialMintBurn>()?;
                json!({"confidentialSupply":v.confidential_supply.to_string(), "decryptableSupply":v.decryptable_supply.to_string(),
                    "supplyElgamalPubkey":v.supply_elgamal_pubkey.to_string(), "pendingBurn":v.pending_burn.to_string()})
            }
            ExtensionType::ScaledUiAmount => {
                let v = state.get_extension::<ext::scaled_ui_amount::ScaledUiAmountConfig>()?;
                ensure!(
                    f64::from(v.multiplier).is_finite() && f64::from(v.new_multiplier).is_finite(),
                    "nonfinite scaled UI multiplier"
                );
                serde_json::to_value(v)?
            }
            ExtensionType::Pausable => config!(ext::pausable::PausableConfig),
            ExtensionType::PausableAccount => marker!(ext::pausable::PausableAccount),
            ExtensionType::PermissionedBurn => {
                config!(ext::permissioned_burn::PermissionedBurnConfig)
            }
            ExtensionType::Uninitialized => bail!("unexpected uninitialized extension"),
        };
        result.push(TokenExtension {
            extension_type: format!("{t:?}"),
            type_id: t.into(),
            config,
        });
    }
    ensure!(
        tlv[pos..].iter().all(|b| *b == 0),
        "nonzero trailing Token-2022 TLV data"
    );
    result.sort_by_key(|e| e.type_id);
    Ok(result)
}
