use crate::sdk::prelude::*;
use crate::{error::VotingError, instructions::create_candidate::CreateCandidateArgs, state::{candidate::Candidate, pull::Pull}};

/// Context for creating new candidate
/// Just like in Anchor, but handcrafted :p
struct CreateCandidateCtx<'info> {
    pub payer: &'info AccountInfo<'info>,
    pub pull: &'info AccountInfo<'info>,
    pub candidate: &'info AccountInfo<'info>,
    pub system_program: &'info AccountInfo<'info>,
}

impl<'info> CreateCandidateCtx<'info> {
    pub fn parse(program_id: &Pubkey, accounts: &'info [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();
        Ok(Self {
            payer: next_account_info(iter)?.require_signer()?.require_mut()?,
            pull: next_account_info(iter)?.require_owner(program_id)?.require_mut()?,
            candidate: next_account_info(iter)?.require_empty()?.require_signer()?.require_mut()?,
            system_program: next_account_info(iter)?.require_system()?,
        })
    }

    pub fn checks(&self) -> ProgramResult {
        let clock = Clock::get()?;
        require!(
            self.pull.load::<Pull>()?.voting_end >= clock.unix_timestamp,
            VotingError::VotingAlreadyEnded,
        );

        Ok(())
    }
}

pub fn create_candidate<'a>(program_id: &Pubkey, accounts: &'a[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    let args = CreateCandidateArgs::parse_from_bytes(data)?;
    let ctx = CreateCandidateCtx::parse(program_id, accounts)?;
    ctx.checks()?;

    ctx.candidate.create_sdk_account_cpi::<Candidate>(&ctx.payer, &ctx.system_program, program_id)?;

    let mut pull_data = ctx.pull.load_mut::<Pull>()?;
    let last_candidate = pull_data.last_candidate;
    pull_data.candidate_count = pull_data.candidate_count.saturating_add(1);
    pull_data.last_candidate = PodOption::some(ctx.candidate.key.clone());

    let mut candidate_data = ctx.candidate.load_mut::<Candidate>()?;
    candidate_data.pull = *ctx.pull.key;
    candidate_data.name = args.name;
    candidate_data.last_candidate = last_candidate;

    Ok(())
}
