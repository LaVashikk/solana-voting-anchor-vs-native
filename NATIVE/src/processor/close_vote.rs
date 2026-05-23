use dummy_sdk::prelude::*;

use crate::{error::VotingError, state::voter::Voter};

struct CloseVoteCtx<'info> {
    pub voter: &'info AccountInfo<'info>, // todo: should i have wrapper for case's like this? why?
    pub bot: SignerAccountMut<'info>,
    pub voter_tracker: OwnedAccountMut<'info, Voter>,
}

impl<'info> CloseVoteCtx<'info> {
    pub fn parse(program_id: &'info Pubkey, accounts: &'info [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();
        Ok(Self {
            voter: next_account_info(iter)?.require_mut()?,
            bot: next_account_info(iter)?.parse_into()?,
            voter_tracker: next_account_info(iter)?.bind_owner(program_id).parse_into()?,
        })
    }

    pub fn checks(&self) -> ProgramResult {
        let clock = Clock::get()?;
        let v_data = self.voter_tracker.load()?;

        require!(
            v_data.voter_addr == *self.voter.key,
            VotingError::InvalidVoter
        );
        require!(
            v_data.voting_end < clock.unix_timestamp,
            VotingError::VotingNotEnded
        );

        Ok(())
    }
}

pub fn close_vote<'a>(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'a>], _data: &[u8]) -> ProgramResult {
    let ctx = CloseVoteCtx::parse(program_id, accounts)?;
    ctx.checks()?;

    let rent = solana_program::rent::Rent::get()?;
    let rent_lamports = rent.minimum_balance(Voter::SIZE);
    let fee = rent_lamports.checked_mul(5).ok_or(VotingError::MathError)? / 100; // 5% fee of rent for bot

    ctx.voter_tracker.send_lamports(fee, &ctx.bot)?;
    ctx.voter_tracker.close(&ctx.voter)
}
