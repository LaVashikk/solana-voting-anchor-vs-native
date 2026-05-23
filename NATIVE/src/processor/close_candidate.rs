use dummy_sdk::prelude::*;

use crate::{error::VotingError, state::{candidate::Candidate, pull::Pull}};

struct CloseCandidateCtx<'info> {
    pub creator: SignerAccountMut<'info>,
    pub pull: OwnedAccountMut<'info, Pull>,
    pub candidate: OwnedAccountMut<'info, Candidate>,
    // pub prev_candidate: OwnedAccountMut<'info, Candidate>,
}

impl<'info> CloseCandidateCtx<'info> {
    pub fn parse(program_id: &'info Pubkey, accounts: &'info [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();
        Ok(Self {
            creator: next_account_info(iter)?.parse_into()?,
            pull: next_account_info(iter)?.bind_owner(program_id).parse_into()?,
            candidate: next_account_info(iter)?.bind_owner(program_id).parse_into()?,
        })
    }

    pub fn checks(&self) -> ProgramResult {
        let pull = self.pull.load()?;
        let candidate = self.candidate.load()?;
        let clock = Clock::get()?;

        require!(
            pull.creator == *self.creator.key,
            VotingError::InvalidCreator
        );
        require!(
            candidate.pull == *self.pull.key,
            VotingError::InvalidPull
        );
        require!(
            pull.last_candidate.map(|c| c == *self.candidate.key) == Some(true),
           VotingError::InvalidCandidate
        );
        require!(
            pull.voting_end < clock.unix_timestamp,
            VotingError::VotingNotEnded
        );

        Ok(())
    }
}

pub fn close_candidate<'a>(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'a>], _data: &[u8]) -> ProgramResult {
    let ctx = CloseCandidateCtx::parse(program_id, accounts)?;
    ctx.checks()?;

    let new_last = ctx.candidate.load()?.prev_candidate;
    ctx.pull.with_mut(|p| {
        p.candidate_count -= 1;
        p.last_candidate = new_last
    })?;

    ctx.candidate.close(&ctx.creator)
}
