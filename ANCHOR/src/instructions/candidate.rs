use anchor_lang::prelude::*;

use crate::{errors::VotingError, state::{Candidate, Pull}};


#[derive(Accounts)]
pub struct CreateCandidate<'info> {
    #[account(mut, address = pull.creator @ VotingError::InvalidCreator)]
    payer : Signer<'info>,

    /// SAFETY: There is no seeds for checking, because account ALREADY created by our program
    /// (Anchor checks it for us), so it's safe AND cheap
    #[account(mut)]
    pull: Account<'info, Pull>,

    #[account(
        init,
        payer = payer,
        seeds = [
            b"candidate",
            pull.key().as_ref(),
            &pull.next_candidate_idx.to_le_bytes()
        ],
        bump,
        space = 8 + Candidate::INIT_SPACE,
    )]
    candidate_pda: Account<'info, Candidate>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseCandidate<'info> {
    #[account(mut, address = pull.creator @ VotingError::InvalidCreator)]
    pub creator: Signer<'info>,

    /// SAFETY: no seeds checking, because account created by our program
    #[account(mut)]
    pub pull: Account<'info, Pull>,

    #[account(mut, close = creator, constraint = candidate.pull_key == pull.key() @ VotingError::InvalidPull)]
    pub candidate: Account<'info, Candidate>,
}


pub fn handle_create_candidate(ctx: Context<CreateCandidate>, candidate_name: String) -> Result<()> {
    let clock = Clock::get()?;
    require!(
        ctx.accounts.pull.metadata.voting_end >= clock.unix_timestamp,
        VotingError::VotingAlreadyEnded,
    );
    require!(candidate_name.len() <= 32, VotingError::NameTooLong);
    // YES, we can add more new candidate even if voting already started
    // feature, not a bug :p

    // Update data in pull
    let pull = &mut ctx.accounts.pull;
    pull.candidate_count = pull.candidate_count.checked_add(1).ok_or(VotingError::MathError)?;
    pull.next_candidate_idx = pull.next_candidate_idx.checked_add(1).ok_or(VotingError::MathError)?;

    let candidate = &mut ctx.accounts.candidate_pda;
    // This 'Compiler-Driven Development' approach more safety, then dot-notation;
    let Candidate {
        pull_key,
        name,
        bump,
        number_of_votes: _,
    } = &mut **candidate;

    *pull_key = pull.key();
    *name = candidate_name;
    *bump = ctx.bumps.candidate_pda;

    Ok(())
}

pub fn handle_close_candidate(ctx: Context<CloseCandidate>) -> Result<()> {
    let clock = Clock::get()?;
    require!(
        ctx.accounts.pull.metadata.voting_end < clock.unix_timestamp,
        VotingError::VotingNotEnded,
    );

    let pull = &mut ctx.accounts.pull;
    pull.candidate_count = pull.candidate_count.checked_sub(1).ok_or(VotingError::MathError)?;

    Ok(())
}
