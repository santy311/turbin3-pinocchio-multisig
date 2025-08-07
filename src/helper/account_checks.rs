use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::helper::utils::DataLen;

#[inline(always)]
pub fn check_signer(account: &AccountInfo) -> Result<(), ProgramError> {
    if !account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

#[inline(always)]
pub fn check_ix_payer_valid(account: &AccountInfo, ix_payer: &Pubkey) -> Result<(), ProgramError> {
    if account.key() != ix_payer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

#[inline(always)]
pub fn check_pda_valid(account: &AccountInfo) -> Result<(), ProgramError> {
    if !account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

#[inline(always)]
pub fn derive_pda_valid(account: &AccountInfo, pubkey: &Pubkey) -> Result<(), ProgramError> {
    if pubkey.ne(account.key()) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    Ok(())
}

 