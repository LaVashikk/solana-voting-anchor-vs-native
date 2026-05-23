use dummy_sdk::prelude::*;

use crate::{error::VotingError, state::pull::Pull};

struct ClosePullCtx<'info> {
    pub creator: SignerAccount<'info>,
    pub pull: OwnedAccountMut<'info, Pull>,
}

impl<'info> ClosePullCtx<'info> {
    pub fn parse(program_id: &'info Pubkey, accounts: &'info [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();
        Ok(Self {
            creator: next_account_info(iter)?.parse_into()?,
            pull: next_account_info(iter)?.bind_owner(program_id).parse_into()?,
        })
    }

    pub fn checks(&self) -> ProgramResult {
        let clock = Clock::get()?;
        let pull = self.pull.load()?;

        require!(
            pull.creator == *self.creator.key,
            VotingError::InvalidCreator
        );
        require!(
            pull.last_candidate.is_none(),
            VotingError::CandidatesNotClosed
        );
        require!(
            pull.voting_end < clock.unix_timestamp,
            VotingError::VotingNotEnded,
        );

        Ok(())
    }
}

pub fn close_pull<'a>(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'a>], _data: &[u8]) -> ProgramResult {
    let ctx = ClosePullCtx::parse(program_id, accounts)?;
    ctx.checks()?;

    ctx.pull.close(&ctx.creator)
}
