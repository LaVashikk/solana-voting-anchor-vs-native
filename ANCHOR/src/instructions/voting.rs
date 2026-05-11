use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use crate::{errors::VotingError, state::{Candidate, Pull, Voter}};

#[derive(Accounts)]
pub struct Voting<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// SAFETY: no seeds checking, because account created by our program
    pub pull: Account<'info, Pull>,

    /// SAFETY: no seeds checking, because account created by our program
    #[account(mut, constraint = candidate.pull_key == pull.key() @ VotingError::InvalidPull)]
    pub candidate: Account<'info, Candidate>,

    #[account(
        init,
        payer = payer,
        seeds = [b"voter", pull.key().as_ref(), payer.key().as_ref()],
        bump,
        space = 8 + Voter::INIT_SPACE
    )]
    pub voter_tracker: Account<'info, Voter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseVoting<'info> {
    /// CHECK: It's safe because we check the creator address for pull
    #[account(mut, address = voter_tracker.voter_addr @ VotingError::InvalidVoter)]
    pub voter: UncheckedAccount<'info>,

    #[account(mut)]
    pub bot: Signer<'info>,

    #[account(mut, close = voter)]
    pub voter_tracker: Account<'info, Voter>,
}


pub fn handle_voting(ctx: Context<Voting>) -> Result<()> {
    let clock = Clock::get()?;
    require!(
        ctx.accounts.pull.metadata.voting_start <= clock.unix_timestamp,
        VotingError::VotingNotStarted,
    );
    require!(
        ctx.accounts.pull.metadata.voting_end >= clock.unix_timestamp,
        VotingError::VotingAlreadyEnded,
    );

    let candidate = &mut ctx.accounts.candidate;
    candidate.number_of_votes = candidate.number_of_votes.checked_add(1).ok_or(VotingError::MathError)?;

    // DOT notation is fine here
    let voter = &mut ctx.accounts.voter_tracker;
    voter.voter_addr = ctx.accounts.payer.key();
    voter.voted_for = ctx.accounts.candidate.key();
    voter.voting_end = ctx.accounts.pull.metadata.voting_end;

    // Transfer 'vote_price' lamports to voter_tracker to help prevent Sybil attacks
    let vote_price = ctx.accounts.pull.vote_price;
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: ctx.accounts.voter_tracker.to_account_info(),
        },
    );

    if vote_price > 0 {
        transfer(cpi_context, vote_price)?;
    }

    Ok(())
}

pub fn handle_close_voting(ctx: Context<CloseVoting>) -> Result<()> {
    let clock = Clock::get()?;
    require!(
        ctx.accounts.voter_tracker.voting_end < clock.unix_timestamp,
        VotingError::VotingNotEnded,
    );

    let voter_info = ctx.accounts.voter_tracker.to_account_info();
    let bot_info = ctx.accounts.bot.to_account_info();

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(8 + Voter::INIT_SPACE);

    let fee = rent_lamports.checked_mul(5).ok_or(VotingError::MathError)? / 100; // 5% fee of rent for bot

    // Transfer fee from voter to bot
    **voter_info.lamports.borrow_mut() = voter_info.lamports().checked_sub(fee).ok_or(VotingError::MathError)?;
    **bot_info.lamports.borrow_mut() = bot_info.lamports().checked_add(fee).ok_or(VotingError::MathError)?;

    // The remaining lamports will be transferred to "voter" by Anchor itself, since `close = voter` in struct CloseVoting
    Ok(())
}
