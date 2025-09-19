use crate::helper::account_init::StateDefinition;
use crate::helper::DataLen;
use crate::state::{member::MemberState, multisig::MultisigState, proposal::ProposalState};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use pinocchio::msg;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, shank::ShankType)]
pub struct VoteIxData {
    pub multisig_bump: u8,
    pub proposal_bump: u8,
    pub vote: u8,
}

impl DataLen for VoteIxData {
    const LEN: usize = core::mem::size_of::<VoteIxData>();
}

impl VoteIxData {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            multisig_bump: data[0],
            proposal_bump: data[1],
            vote: data[2],
        })
    }

    pub fn to_bytes(&self) -> Result<[u8; Self::LEN], ProgramError> {
        let mut bytes = [0u8; Self::LEN];
        bytes[0] = self.multisig_bump;
        bytes[1] = self.proposal_bump;
        bytes[2] = self.vote;
        Ok(bytes)
    }
}

pub fn process_vote_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [voter, multisig_account, proposal_account, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let ix_data = VoteIxData::from_bytes(data)?;

    let (_, members) = unsafe {
        multisig_account
            .borrow_mut_data_unchecked()
            .split_at_mut_unchecked(MultisigState::LEN)
    };

    let mut member_exists: Option<MemberState> = None;
    for (_, m) in members.chunks_exact(MemberState::LEN).enumerate() {
        let member = MemberState::from_bytes(m)?;
        if member.pubkey == *voter.key() {
            member_exists = Some(member);
            break;
        }
    }

    if member_exists.is_none() {
        // Member not a part of the multisig
        return Err(ProgramError::InvalidInstructionData);
    }

    let (proposal, votes) = unsafe {
        proposal_account
            .borrow_mut_data_unchecked()
            .split_at_mut_unchecked(ProposalState::LEN)
    };

    let mut proposal = ProposalState::from_bytes(proposal)?;

    let mut voted = false;
    let mut vote_index = 0;
    for (i, vote) in votes.chunks_exact(32).enumerate() {
        if let Some(member) = member_exists {
            if vote == member.pubkey.as_ref() {
                if i < proposal.yes_votes as usize && ix_data.vote == 1 {
                    // Already voted yes
                    return Err(ProgramError::InvalidInstructionData);
                } else if proposal.yes_votes as usize <= i && ix_data.vote == 0 {
                    // Already voted no
                    return Err(ProgramError::InvalidInstructionData);
                }
                voted = true;
                vote_index = i;
                break;
            }
        }
    }

    if voted {
        // Swap and update the vote count
        if ix_data.vote == 1 {
            // replace the last yes vote with current vote and reduce the yes vote count and increase the no vote count
            let vote_start = vote_index * 32;
            let swap_start = proposal.yes_votes as usize * 32;

            if vote_start < swap_start {
                let (left, right) = votes.split_at_mut(swap_start);
                left[vote_start..vote_start + 32].swap_with_slice(&mut right[0..32]);
            } else {
                let (left, right) = votes.split_at_mut(vote_start);
                left[swap_start..swap_start + 32].swap_with_slice(&mut right[0..32]);
            }

            proposal.yes_votes -= 1;
            proposal.no_votes += 1;
        } else {
            // replace the last no vote with current vote and increase the yes vote count and reduce the no vote count
            let vote_start = vote_index * 32;
            let swap_start = proposal.no_votes as usize * 32;

            if vote_start < swap_start {
                let (left, right) = votes.split_at_mut(swap_start);
                left[vote_start..vote_start + 32].swap_with_slice(&mut right[0..32]);
            } else {
                let (left, right) = votes.split_at_mut(vote_start);
                left[swap_start..swap_start + 32].swap_with_slice(&mut right[0..32]);
            }

            proposal.yes_votes += 1;
            proposal.no_votes -= 1;
        }
    } else {
        msg!("dbg1");
        // Increase the size of the account to add new vote
        let new_size = proposal_account.data_len() + 32;
        let rent_diff = Rent::get()?.minimum_balance(new_size) - proposal_account.lamports();

        msg!("dbg2");

        unsafe {
            *voter.borrow_mut_lamports_unchecked() -= rent_diff;
            *proposal_account.borrow_mut_lamports_unchecked() += rent_diff;
        }

        msg!("dbg3");

        proposal_account.resize(new_size)?;

        msg!("dbg4");

        let (new_proposal, new_votes) = unsafe {
            proposal_account
                .borrow_mut_data_unchecked()
                .split_at_mut_unchecked(ProposalState::LEN)
        };

        msg!("dbg5");

        // Update the proposal data directly
        let proposal_ptr = new_proposal.as_mut_ptr() as *mut ProposalState;

        msg!("dbg6");

        unsafe {
            *proposal_ptr = proposal;
        }

        msg!("dbg7");

        if let Some(member) = member_exists {
            if ix_data.vote == 1 {
                // swap the last yes vote with first no vote
                msg!("dbg8");
                let yes_start = proposal.yes_votes as usize * 32;
                let no_start = proposal.no_votes as usize * 32;

                if yes_start < no_start {
                    msg!("dbg9");
                    let (left, right) = new_votes.split_at_mut(no_start);
                    left[yes_start..yes_start + 32].swap_with_slice(&mut right[0..32]);
                } else {
                    msg!("dbg10");
                    let (left, right) = new_votes.split_at_mut(yes_start);
                    left[no_start..no_start + 32].swap_with_slice(&mut right[0..32]);
                }

                msg!("dbg11");

                proposal.yes_votes += 1;
            } else {
                // Add the new no vote
                msg!("dbg12");
                let no_start = proposal.no_votes as usize * 32;
                new_votes[no_start..no_start + 32].copy_from_slice(member.pubkey.as_ref());
                proposal.no_votes += 1;
                msg!("dbg13");
            }
        }
    }

    Ok(())
}
