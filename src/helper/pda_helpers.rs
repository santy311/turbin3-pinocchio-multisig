use pinocchio::{
    account_info::AccountInfo,
    pubkey::{self, Pubkey},
    program_error::ProgramError,
};
use crate::helper::utils::DataLen;

pub trait HasSeed {
    const SEED: &'static str;
}

#[inline(always)]
pub fn from_account_info_unchecked<T: DataLen>(account_info: &AccountInfo) -> &mut T {
    unsafe { &mut *(account_info.borrow_mut_data_unchecked().as_ptr() as *mut T) }
}

#[inline(always)]
pub fn from_account_info<T: DataLen>(account_info: &AccountInfo) -> Result<&mut T, pinocchio::program_error::ProgramError> {
    if account_info.data_len() < T::LEN {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }
    Ok(from_account_info_unchecked(account_info))
}

#[inline(always)]
pub fn validate_pda<T: DataLen + HasSeed>(bump: u8, pda: &Pubkey, owner: &Pubkey) -> Result<(), ProgramError> {
    let seed_with_bump = &[T::SEED.as_bytes(), owner, &[bump]];
    let (derived, _) = pubkey::find_program_address(seed_with_bump, &crate::ID);
    if derived != *pda {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}