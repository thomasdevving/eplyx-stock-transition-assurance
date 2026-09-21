//! Eplyx Demo Candidate Conversion.
//!
//! A deliberately small, registered **candidate** mechanism: an operator says
//! "this is the conversion I intend to deploy", and Eplyx executes exactly this
//! program against freshly captured current holder state before any rollout.
//!
//! It converts an old token into a replacement token at a fixed rational ratio:
//! the holder's source tokens are burned through the real deployed token program,
//! and replacement tokens are released from a *proposed* reserve vault owned by a
//! candidate program-derived authority. The ratio, rounding and conversion fee are
//! read from this program's own config account and enforced here, in checked
//! integer arithmetic, so a host-side calculation can never stand in for execution.
//!
//! It is **not** an issuer mechanism, is not deployed on any cluster, and holds no
//! issuer authority: the reserve authority is a candidate PDA of this program, never
//! a real mint authority. Executing it proves that the supplied candidate plan works
//! under the declared authority model, never that an issuer defined or controls it.

#![deny(unsafe_code)]

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

solana_program::declare_id!("He4VZWmVgtbXVmHJ3tRmbLKuNDo9WG3tw5Gr36KupJUf");

/// Exact serialized length of the candidate configuration account.
pub const CONFIG_LEN: usize = 151;
/// Only instruction: convert an exact raw source amount.
pub const CONVERT_TAG: u8 = 1;
/// Seed of the candidate reserve authority, derived per configuration account.
pub const VAULT_SEED: &[u8] = b"eplyx-candidate-vault";
/// Seed of the candidate configuration account, derived per plan digest.
pub const CONFIG_SEED: &[u8] = b"eplyx-candidate-config";
/// SPL Token `BurnChecked` discriminant, shared by both token programs.
pub const BURN_CHECKED: u8 = 15;
/// SPL Token `TransferChecked` discriminant, shared by both token programs.
pub const TRANSFER_CHECKED: u8 = 12;

/// Candidate-mechanism failures. Codes are stable so the host can name them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ConversionError {
    MalformedInstruction = 1,
    ZeroAmount = 2,
    ConfigNotOwned = 3,
    ConfigLayout = 4,
    MintMismatch = 5,
    ReserveMismatch = 6,
    VaultAuthorityMismatch = 7,
    HolderNotSigner = 8,
    TokenProgramMismatch = 9,
    SourceAccountMismatch = 10,
    SourceNotConsumedExactly = 11,
    ZeroOutput = 12,
    InsufficientReserve = 13,
    ArithmeticOverflow = 14,
    ReserveAccountMismatch = 15,
    DestinationMismatch = 16,
    ReserveNotReleasedExactly = 17,
    UnsupportedRounding = 18,
    InvalidTerms = 19,
}
impl ConversionError {
    pub fn name(code: u32) -> Option<&'static str> {
        Some(match code {
            1 => "MalformedInstruction",
            2 => "ZeroAmount",
            3 => "ConfigNotOwned",
            4 => "ConfigLayout",
            5 => "MintMismatch",
            6 => "ReserveMismatch",
            7 => "VaultAuthorityMismatch",
            8 => "HolderNotSigner",
            9 => "TokenProgramMismatch",
            10 => "SourceAccountMismatch",
            11 => "SourceNotConsumedExactly",
            12 => "ZeroOutput",
            13 => "InsufficientReserve",
            14 => "ArithmeticOverflow",
            15 => "ReserveAccountMismatch",
            16 => "DestinationMismatch",
            17 => "ReserveNotReleasedExactly",
            18 => "UnsupportedRounding",
            19 => "InvalidTerms",
            _ => return None,
        })
    }
}
fn fail(error: ConversionError) -> ProgramError {
    ProgramError::Custom(error as u32)
}

/// Exact candidate terms, decoded from the configuration account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Terms {
    pub ratio_numerator: u64,
    pub ratio_denominator: u64,
    /// 0 = Floor, 1 = Ceiling.
    pub rounding: u8,
    pub conversion_fee_bps: u16,
}

/// The whole economic rule of this mechanism, in checked integer arithmetic.
///
/// The conversion fee is taken from the consumed source amount first; the ratio
/// then applies to what remains. Token-2022 transfer fees are a different thing
/// entirely and are never folded in here.
pub fn convert_amount(consumed: u64, terms: &Terms) -> Result<(u64, u64), ConversionError> {
    if terms.ratio_numerator == 0
        || terms.ratio_denominator == 0
        || terms.conversion_fee_bps > 10_000
    {
        return Err(ConversionError::InvalidTerms);
    }
    if terms.rounding > 1 {
        return Err(ConversionError::UnsupportedRounding);
    }
    let consumed = u128::from(consumed);
    let fee = consumed
        .checked_mul(u128::from(terms.conversion_fee_bps))
        .ok_or(ConversionError::ArithmeticOverflow)?
        / 10_000;
    let base = consumed
        .checked_sub(fee)
        .ok_or(ConversionError::ArithmeticOverflow)?;
    let numerator = base
        .checked_mul(u128::from(terms.ratio_numerator))
        .ok_or(ConversionError::ArithmeticOverflow)?;
    let denominator = u128::from(terms.ratio_denominator);
    let output = if terms.rounding == 0 {
        numerator / denominator
    } else {
        numerator
            .checked_add(denominator - 1)
            .ok_or(ConversionError::ArithmeticOverflow)?
            / denominator
    };
    if output > u128::from(u64::MAX) || fee > u128::from(u64::MAX) {
        return Err(ConversionError::ArithmeticOverflow);
    }
    Ok((fee as u64, output as u64))
}

fn key_at(data: &[u8], offset: usize) -> Result<Pubkey, ProgramError> {
    let bytes: [u8; 32] = data
        .get(offset..offset + 32)
        .ok_or_else(|| fail(ConversionError::ConfigLayout))?
        .try_into()
        .map_err(|_| fail(ConversionError::ConfigLayout))?;
    Ok(Pubkey::new_from_array(bytes))
}

/// Base SPL token-account layout, shared by the legacy program and Token-2022.
fn token_amount(data: &[u8]) -> Result<u64, ProgramError> {
    let bytes: [u8; 8] = data
        .get(64..72)
        .ok_or_else(|| fail(ConversionError::SourceAccountMismatch))?
        .try_into()
        .map_err(|_| fail(ConversionError::SourceAccountMismatch))?;
    Ok(u64::from_le_bytes(bytes))
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if data.len() != 9 || data[0] != CONVERT_TAG {
        return Err(fail(ConversionError::MalformedInstruction));
    }
    let amount = u64::from_le_bytes(
        data[1..9]
            .try_into()
            .map_err(|_| fail(ConversionError::MalformedInstruction))?,
    );
    if amount == 0 {
        return Err(fail(ConversionError::ZeroAmount));
    }
    let infos = &mut accounts.iter();
    let config = next_account_info(infos)?;
    let source = next_account_info(infos)?;
    let source_mint = next_account_info(infos)?;
    let holder = next_account_info(infos)?;
    let reserve = next_account_info(infos)?;
    let replacement_mint = next_account_info(infos)?;
    let destination = next_account_info(infos)?;
    let vault_authority = next_account_info(infos)?;
    let source_program = next_account_info(infos)?;
    let replacement_program = next_account_info(infos)?;

    if config.owner != program_id {
        return Err(fail(ConversionError::ConfigNotOwned));
    }
    let (terms, source_decimals, replacement_decimals, bump) = {
        let cfg = config.try_borrow_data()?;
        if cfg.len() != CONFIG_LEN || cfg[0] != 1 {
            return Err(fail(ConversionError::ConfigLayout));
        }
        if key_at(&cfg, 1)? != *source_mint.key || key_at(&cfg, 33)? != *replacement_mint.key {
            return Err(fail(ConversionError::MintMismatch));
        }
        if key_at(&cfg, 65)? != *reserve.key {
            return Err(fail(ConversionError::ReserveMismatch));
        }
        if key_at(&cfg, 97)? != *vault_authority.key {
            return Err(fail(ConversionError::VaultAuthorityMismatch));
        }
        let read_u64 = |offset: usize| -> Result<u64, ProgramError> {
            let bytes: [u8; 8] = cfg[offset..offset + 8]
                .try_into()
                .map_err(|_| fail(ConversionError::ConfigLayout))?;
            Ok(u64::from_le_bytes(bytes))
        };
        (
            Terms {
                ratio_numerator: read_u64(129)?,
                ratio_denominator: read_u64(137)?,
                rounding: cfg[145],
                conversion_fee_bps: u16::from_le_bytes([cfg[146], cfg[147]]),
            },
            cfg[148],
            cfg[149],
            cfg[150],
        )
    };
    let expected =
        Pubkey::create_program_address(&[VAULT_SEED, config.key.as_ref(), &[bump]], program_id)
            .map_err(|_| fail(ConversionError::VaultAuthorityMismatch))?;
    if expected != *vault_authority.key {
        return Err(fail(ConversionError::VaultAuthorityMismatch));
    }
    if !holder.is_signer {
        return Err(fail(ConversionError::HolderNotSigner));
    }
    if source.owner != source_program.key
        || source_mint.owner != source_program.key
        || reserve.owner != replacement_program.key
        || destination.owner != replacement_program.key
        || replacement_mint.owner != replacement_program.key
    {
        return Err(fail(ConversionError::TokenProgramMismatch));
    }
    let source_before = {
        let data = source.try_borrow_data()?;
        if key_at(&data, 0)? != *source_mint.key || key_at(&data, 32)? != *holder.key {
            return Err(fail(ConversionError::SourceAccountMismatch));
        }
        token_amount(&data)?
    };
    {
        let data = reserve.try_borrow_data()?;
        if key_at(&data, 0)? != *replacement_mint.key || key_at(&data, 32)? != *vault_authority.key
        {
            return Err(fail(ConversionError::ReserveAccountMismatch));
        }
    }
    {
        let data = destination.try_borrow_data()?;
        if key_at(&data, 0)? != *replacement_mint.key {
            return Err(fail(ConversionError::DestinationMismatch));
        }
    }
    if source_before < amount {
        return Err(fail(ConversionError::SourceNotConsumedExactly));
    }

    let burn = Instruction {
        program_id: *source_program.key,
        accounts: vec![
            AccountMeta::new(*source.key, false),
            AccountMeta::new(*source_mint.key, false),
            AccountMeta::new_readonly(*holder.key, true),
        ],
        data: [
            &[BURN_CHECKED][..],
            &amount.to_le_bytes()[..],
            &[source_decimals][..],
        ]
        .concat(),
    };
    invoke(
        &burn,
        &[
            source.clone(),
            source_mint.clone(),
            holder.clone(),
            source_program.clone(),
        ],
    )?;
    let consumed = source_before
        .checked_sub(token_amount(&source.try_borrow_data()?)?)
        .ok_or_else(|| fail(ConversionError::SourceNotConsumedExactly))?;
    if consumed != amount {
        return Err(fail(ConversionError::SourceNotConsumedExactly));
    }

    let (conversion_fee, output) = convert_amount(consumed, &terms).map_err(fail)?;
    if output == 0 {
        return Err(fail(ConversionError::ZeroOutput));
    }
    let reserve_before = token_amount(&reserve.try_borrow_data()?)?;
    if reserve_before < output {
        return Err(fail(ConversionError::InsufficientReserve));
    }
    let release = Instruction {
        program_id: *replacement_program.key,
        accounts: vec![
            AccountMeta::new(*reserve.key, false),
            AccountMeta::new_readonly(*replacement_mint.key, false),
            AccountMeta::new(*destination.key, false),
            AccountMeta::new_readonly(*vault_authority.key, true),
        ],
        data: [
            &[TRANSFER_CHECKED][..],
            &output.to_le_bytes()[..],
            &[replacement_decimals][..],
        ]
        .concat(),
    };
    invoke_signed(
        &release,
        &[
            reserve.clone(),
            replacement_mint.clone(),
            destination.clone(),
            vault_authority.clone(),
            replacement_program.clone(),
        ],
        &[&[VAULT_SEED, config.key.as_ref(), &[bump]]],
    )?;
    let reserve_after = token_amount(&reserve.try_borrow_data()?)?;
    if reserve_before.checked_sub(reserve_after) != Some(output) {
        return Err(fail(ConversionError::ReserveNotReleasedExactly));
    }
    msg!(
        "EPLYX_CANDIDATE_CONVERSION consumed={} conversion_fee={} released={} ratio={}/{} rounding={}",
        consumed,
        conversion_fee,
        output,
        terms.ratio_numerator,
        terms.ratio_denominator,
        terms.rounding
    );
    Ok(())
}

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(entry);

#[cfg(not(feature = "no-entrypoint"))]
fn entry(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    process_instruction(program_id, accounts, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terms(n: u64, d: u64, rounding: u8, bps: u16) -> Terms {
        Terms {
            ratio_numerator: n,
            ratio_denominator: d,
            rounding,
            conversion_fee_bps: bps,
        }
    }
    #[test]
    fn floor_and_ceiling_rounding_are_exact() {
        assert_eq!(convert_amount(7, &terms(1, 2, 0, 0)), Ok((0, 3)));
        assert_eq!(convert_amount(7, &terms(1, 2, 1, 0)), Ok((0, 4)));
        assert_eq!(convert_amount(1000, &terms(3, 7, 0, 0)), Ok((0, 428)));
        assert_eq!(convert_amount(1000, &terms(3, 7, 1, 0)), Ok((0, 429)));
    }
    #[test]
    fn conversion_fee_precedes_the_ratio() {
        assert_eq!(convert_amount(1000, &terms(1, 1, 0, 250)), Ok((25, 975)));
        assert_eq!(convert_amount(1000, &terms(2, 1, 0, 10_000)), Ok((1000, 0)));
    }
    #[test]
    fn invalid_terms_and_overflow_are_rejected() {
        assert_eq!(
            convert_amount(1, &terms(1, 0, 0, 0)),
            Err(ConversionError::InvalidTerms)
        );
        assert_eq!(
            convert_amount(1, &terms(0, 1, 0, 0)),
            Err(ConversionError::InvalidTerms)
        );
        assert_eq!(
            convert_amount(1, &terms(1, 1, 0, 10_001)),
            Err(ConversionError::InvalidTerms)
        );
        assert_eq!(
            convert_amount(1, &terms(1, 1, 2, 0)),
            Err(ConversionError::UnsupportedRounding)
        );
        assert_eq!(
            convert_amount(u64::MAX, &terms(u64::MAX, 1, 0, 0)),
            Err(ConversionError::ArithmeticOverflow)
        );
    }
}
