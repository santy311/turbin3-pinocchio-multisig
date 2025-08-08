#![allow(unexpected_cfgs)]
// #![no_std]

#[cfg(feature = "std")]
extern crate std;

use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

pub mod helper;
pub mod instructions;
pub mod state;

use instructions::*;

entrypoint!(process_instruction);

pinocchio_pubkey::declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, _) = data.split_first().ok_or(ProgramError::InvalidAccountData)?;

    match MultisigInstructions::try_from(discriminator)? {
        MultisigInstructions::InitMultisig => {
            instructions::process_init_multisig_instruction(accounts, data)?
        }
        MultisigInstructions::CreateTransaction => {
            instructions::process_create_transaction(accounts, data)?
        } //MultisigInstructions::UpdateMultisig => instructions::process_init_multisig_instruction(accounts, data)?,
          //MultisigInstructions::CreateProposal => instructions::process_init_multisig_instruction(accounts, data)?,
          //MultisigInstructions::Vote => instructions::process_init_multisig_instruction(accounts, data)?,
    }

    Ok(())
}
