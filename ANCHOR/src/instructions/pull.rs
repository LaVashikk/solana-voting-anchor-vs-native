use anchor_lang::prelude::*;

use crate::{errors::VotingError, state::{Pull, PullMetadata}};

#[derive(Accounts)]
#[instruction(metadata: PullMetadata)]
pub struct CreatePull<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [b"pull", metadata.title.as_bytes()], // TODO: change it! Use global counter
        bump,
        space = 8 + Pull::INIT_SPACE
    )]
    pull_pda: Account<'info, Pull>,
    #[account(mut)]
    payer : Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePull<'info> {
    #[account(mut, address = pull.creator @ VotingError::InvalidCreator)]
    pub creator: Signer<'info>,

    /// SAFETY: no seeds checking, because account created by our program
    #[account(mut, close = creator, constraint = pull.candidate_count == 0 @ VotingError::CandidatesNotClosed)]
    pub pull: Account<'info, Pull>,
}

pub fn handle_create_pull(ctx: Context<CreatePull>, metadata: PullMetadata, vote_price: u64) -> Result<()> {
    require!(metadata.title.len() <= 32, VotingError::NameTooLong);
    require!(metadata.description.len() <= 128, VotingError::DescTooLong);
    require!(metadata.voting_start < metadata.voting_end, VotingError::InvalidTimeRange);
    require!(metadata.voting_end > Clock::get()?.unix_timestamp, VotingError::VotingAlreadyEnded);

    let pull = &mut ctx.accounts.pull_pda;

    // I really more like this Compiler-Driven Development approach, instead dot-notation;
    // because it guarantees compile-time safety against unhandled state fields.
    // Any future additions to the `Pull` account will break the build here
    let Pull {
        creator,
        metadata: p_meta,
        vote_price: v_price,
        bump,
        next_candidate_idx: _,
        candidate_count: _,
    } = &mut **pull;

    *creator = ctx.accounts.payer.key();
    *p_meta = metadata;
    *v_price = vote_price;
    *bump = ctx.bumps.pull_pda;

    Ok(())
}

pub fn handle_close_pull(ctx: Context<ClosePull>) -> Result<()> {
    let clock = Clock::get()?;
    require!(
        ctx.accounts.pull.metadata.voting_end < clock.unix_timestamp,
        VotingError::VotingNotEnded,
    );

    Ok(())
}
