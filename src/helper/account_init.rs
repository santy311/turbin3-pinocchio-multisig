use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::helper::utils::DataLen;
use crate::helper::pda_helpers::HasSeed;

pub trait HasOwner {
    fn owner(&self) -> &Pubkey;
}

pub trait StateDefinition {
    const LEN: usize;
    const SEED: &'static str;
}

#[inline(always)]
pub fn create_pda_account<S, I>(
    payer: &AccountInfo,
    account: &AccountInfo,
    ix_data: &I,
    rent: &Rent,
) -> Result<u8, ProgramError>
where
    S: StateDefinition,
    I: HasOwner,
{
    let (derived_pda, bump) = pubkey::find_program_address(
        &[S::SEED.as_bytes(), ix_data.owner().as_ref()],
        &crate::ID,
    );

    if derived_pda != *account.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    let bump_bytes = [bump];
    let signer_seeds = [
        Seed::from(S::SEED.as_bytes()),
        Seed::from(ix_data.owner()),
        Seed::from(&bump_bytes[..]),
    ];
    let signers = [Signer::from(&signer_seeds[..])];

    CreateAccount {
        from: payer,
        to: account,
        space: S::LEN as u64,
        owner: &crate::ID,
        lamports: rent.minimum_balance(S::LEN),
    }
    .invoke_signed(&signers)?;

    Ok(bump)
}

#[inline(always)]
pub fn create_pda_account_with_custom_seeds<S>(
    payer: &AccountInfo,
    account: &AccountInfo,
    signer_seeds: &[Seed],
    rent: &Rent,
) -> Result<(), ProgramError>
where
    S: StateDefinition,
{
    let signers = [Signer::from(signer_seeds)];

    CreateAccount {
        from: payer,
        to: account,
        space: S::LEN as u64,
        owner: &crate::ID,
        lamports: rent.minimum_balance(S::LEN),
    }
    .invoke_signed(&signers)?;

    Ok(())
}
