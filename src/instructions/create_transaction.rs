use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::rent::Rent,
    ProgramResult,
};
// use pinocchio_system::instructions::CreateAccount;
use crate::helper::account_init::StateDefinition;
use crate::{
    helper::{
        account_checks::{check_ix_payer_valid, check_pda_valid, check_signer, derive_pda_valid},
        account_init::{create_pda_account, HasOwner},
    },
    state::{utils::DataLen, TransactionState},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreateTransactionIxData {
    pub transaction_index: u64, // 8 bytes
    pub tx_maker: [u8; 32],     // 32 bytes
    pub tx_buffer: [u8; 512],   // 512 bytes
    pub buffer_size: u16,       // 2 bytes
}

impl DataLen for CreateTransactionIxData {
    const LEN: usize = 1 + 8 + 32 + 512 + 2; // Includes discriminator
}

impl HasOwner for CreateTransactionIxData {
    fn owner(&self) -> &Pubkey {
        &self.tx_maker
    }
}

impl CreateTransactionIxData {
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0; Self::LEN];
        bytes[0] = 2;
        bytes[1..9].copy_from_slice(&self.transaction_index.to_le_bytes());
        bytes[9..41].copy_from_slice(&self.tx_maker);
        bytes[41..553].copy_from_slice(&self.tx_buffer);
        bytes[553..555].copy_from_slice(&self.buffer_size.to_le_bytes());
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self {
            transaction_index: u64::from_le_bytes(
                data[1..9]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            ),
            tx_maker: data[9..41]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
            tx_buffer: data[41..553]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
            buffer_size: u16::from_le_bytes(
                data[553..555]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            ),
        })
    }
}

pub fn process_create_transaction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer, transaction_acc, sysvar_rent_acc, _system_program, _rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(&payer)?;

    check_pda_valid(&transaction_acc)?;

    let rent = Rent::from_account_info(sysvar_rent_acc)?;

    let ix_data = CreateTransactionIxData::from_bytes(&data)?;
    check_ix_payer_valid(&payer, &ix_data.tx_maker)?;

    let seeds = &[TransactionState::SEED.as_bytes(), &ix_data.tx_maker];

    let (derived_transaction_pda, bump) = pubkey::find_program_address(seeds, &crate::ID);
    derive_pda_valid(&transaction_acc, &derived_transaction_pda)?;

    create_pda_account::<TransactionState, CreateTransactionIxData>(
        &payer,
        &transaction_acc,
        &ix_data,
        &rent,
    )?;

    // let pda_bump_bytes = [bump];
    // let signer_seeds = [
    //     Seed::from(TransactionState::SEED.as_bytes()),
    //     Seed::from(&ix_data.tx_maker),
    //     Seed::from(&pda_bump_bytes[..]),
    // ];
    // let signers = [Signer::from(&signer_seeds[..])];

    // CreateAccount{
    //     from: payer,
    //     to: transaction_acc,
    //     lamports: rent.minimum_balance(TransactionState::LEN),
    //     space: TransactionState::LEN as u64,
    //     owner: &crate::ID,
    // }.invoke_signed(&signers)?;

    TransactionState::initialize(transaction_acc, &ix_data, bump)?;

    Ok(())
}
