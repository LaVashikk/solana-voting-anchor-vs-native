use dummy_sdk::prelude::*;
use crate::{error::VotingError, state::{candidate::Candidate, voter::Voter, pull::Pull}};

struct VotingCtx<'info> {
    pub payer: &'info AccountInfo<'info>,
    pub pull: &'info AccountInfo<'info>,
    pub candidate: &'info AccountInfo<'info>,
    pub voter_pda: &'info AccountInfo<'info>,
    pub system_program: &'info AccountInfo<'info>,
}

impl<'info> VotingCtx<'info> {
    pub fn parse(program_id: &Pubkey, accounts: &'info [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();

        Ok(Self {
            payer: next_account_info(iter)?.require_signer()?.require_mut()?,
            pull: next_account_info(iter)?.require_owner(program_id)?,
            candidate: next_account_info(iter)?.require_owner(program_id)?.require_mut()?,
            voter_pda: next_account_info(iter)?.require_mut()?.require_empty()?,
            system_program: next_account_info(iter)?.require_system()?,
        })
    }

    pub fn checks(&self) -> ProgramResult {
        let clock = Clock::get()?;
        let pull = self.pull.load::<Pull>()?;
        let candidate = self.candidate.load::<Candidate>()?;

        require!(
            candidate.pull == *self.pull.key,
            VotingError::InvalidPull
        );
        require!(
            pull.voting_start <= clock.unix_timestamp,
            VotingError::VotingNotStarted,
        );
        require!(
            pull.voting_end >= clock.unix_timestamp,
            VotingError::VotingAlreadyEnded,
        );

        Ok(())
    }
}


pub fn voting<'a>(program_id: &Pubkey, accounts: &'a[AccountInfo<'a>], _data: &[u8]) -> ProgramResult {
    let ctx = VotingCtx::parse(program_id, accounts)?;
    ctx.checks()?;

    let seeds = Voter::get_seeds(ctx.pull.key, ctx.payer.key);
    let _bump = ctx.voter_pda.checked_create_sdk_pda_cpi::<Voter>(
        ctx.payer,
        ctx.system_program,
        seeds.as_ref(),
        program_id
    )?;

    ctx.candidate.with_mut::<Candidate, _>(|candidate| {
        candidate.number_of_votes += 1; // it's safe, u64 overflow is economically impossible
    })?;

    ctx.voter_pda.with_mut::<Voter, _>(|voter| {
        voter.voter_addr = *ctx.payer.key;
        voter.voted_for = *ctx.candidate.key;
    })?;

    // Transfer lamports from payer to voter_pda
    let vote_price = ctx.pull.load::<Pull>()?.vote_price;
    if vote_price > 0 {
        ctx.voter_pda.receive_lamports_cpi(&ctx.payer, &ctx.system_program, vote_price)?;
    }

    Ok(())
}
