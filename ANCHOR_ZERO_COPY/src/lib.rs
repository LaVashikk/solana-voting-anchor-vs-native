use std::mem;

use anchor_lang::prelude::*;
use anchor_lang::system_program::{Transfer, transfer};

pub mod state;
pub mod errors;

pub use state::*;
pub use errors::*;

declare_id!("9AvUNHjxscdkiKQ8tUn12QCMXtcnbR9BVGq3ULNzFMRi");

#[program]
pub mod zero_copy_anchor_vote {
    use super::*;

    pub fn create_pull(
        ctx: Context<CreatePull>,
        title: String,
        description: String,
        voting_start: i64,
        voting_end: i64,
        vote_price: u64,
    ) -> Result<()> {
        require!(voting_start < voting_end, VotingError::InvalidTimeRange);
        require!(voting_end > Clock::get()?.unix_timestamp, VotingError::VotingAlreadyEnded);

        let mut pull = ctx.accounts.pull.load_init()?;
        pull.creator = ctx.accounts.payer.key();
        pull.title = dummy_sdk::pod_types::FixedString::try_from(title).map_err(|_| VotingError::NameTooLong)?;
        pull.description = dummy_sdk::pod_types::FixedString::try_from(description).map_err(|_| VotingError::DescTooLong)?;
        pull.voting_start = voting_start;
        pull.voting_end = voting_end;
        pull.vote_price = vote_price;
        pull.candidate_count = 0;
        pull.last_candidate = dummy_sdk::pod_types::PodOption::none();
        pull.next_candidate_idx = 0;

        Ok(())
    }

    pub fn create_candidate(ctx: Context<CreateCandidate>, name: String) -> Result<()> {
        let clock = Clock::get()?;
        let mut pull = ctx.accounts.pull.load_mut()?;

        require!(
            pull.voting_end >= clock.unix_timestamp,
            VotingError::VotingAlreadyEnded,
        );
        require!(
            ctx.accounts.payer.key() == pull.creator,
            VotingError::InvalidCreator
        );

        let last_candidate = pull.last_candidate;
        pull.candidate_count += 1; // u64, we can do what we want here, no checks needed
        pull.last_candidate = dummy_sdk::pod_types::PodOption::some(ctx.accounts.candidate.key());
        pull.next_candidate_idx += 1;

        let mut candidate = ctx.accounts.candidate.load_init()?;
        candidate.pull = ctx.accounts.pull.key();
        candidate.name = dummy_sdk::pod_types::FixedString::try_from(name).map_err(|_| VotingError::NameTooLong)?;
        candidate.prev_candidate = last_candidate;
        candidate.number_of_votes = 0;

        Ok(())
    }

    pub fn voting(ctx: Context<Voting>) -> Result<()> {
        let clock = Clock::get()?;
        let pull = ctx.accounts.pull.load()?;
        let vote_price = pull.vote_price;
        let mut candidate = ctx.accounts.candidate.load_mut()?;

        require!(
            candidate.pull == ctx.accounts.pull.key(),
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

        candidate.number_of_votes += 1; // no checks, overflow is economically impossible

        {
            let mut voter = ctx.accounts.voter_tracker.load_init()?;
            voter.voter_addr = ctx.accounts.payer.key();
            voter.voted_for = ctx.accounts.candidate.key();
            voter.voting_end = pull.voting_end;
        }

        if vote_price > 0 {
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.key(),
                Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.voter_tracker.to_account_info(),
                },
            );
            transfer(cpi_context, vote_price)?;
        }

        Ok(())
    }

    pub fn close_pull(ctx: Context<ClosePull>) -> Result<()> {
        let clock = Clock::get()?;
        let pull = ctx.accounts.pull.load()?;

        require!(
            pull.creator == ctx.accounts.creator.key(),
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

    pub fn close_candidate(ctx: Context<CloseCandidate>) -> Result<()> {
        let clock = Clock::get()?;
        let mut pull = ctx.accounts.pull.load_mut()?;
        let candidate = ctx.accounts.candidate.load()?;

        require!(
            pull.creator == ctx.accounts.creator.key(),
            VotingError::InvalidCreator
        );
        require!(
            candidate.pull == ctx.accounts.pull.key(),
            VotingError::InvalidPull
        );
        require!(
            pull.last_candidate.map(|c| c == ctx.accounts.candidate.key()) == Some(true),
            VotingError::InvalidCandidate
        );
        require!(
            pull.voting_end < clock.unix_timestamp,
            VotingError::VotingNotEnded
        );

        pull.candidate_count -= 1;
        pull.last_candidate = candidate.prev_candidate;

        Ok(())
    }

    pub fn close_vote(ctx: Context<CloseVote>) -> Result<()> {
        let clock = Clock::get()?;
        let voter_data = ctx.accounts.voter_tracker.load()?;

        require!(
            voter_data.voter_addr == ctx.accounts.voter.key(),
            VotingError::InvalidVoter
        );
        require!(
            voter_data.voting_end < clock.unix_timestamp,
            VotingError::VotingNotEnded
        );

        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(8 + mem::size_of::<Voter>());
        let fee = rent_lamports.checked_mul(5).ok_or(VotingError::MathError)? / 100;

        let voter_tracker_info = ctx.accounts.voter_tracker.to_account_info();
        let bot_info = ctx.accounts.bot.to_account_info();

        **voter_tracker_info.lamports.borrow_mut() = voter_tracker_info.lamports().checked_sub(fee).ok_or(VotingError::MathError)?;
        **bot_info.lamports.borrow_mut() = bot_info.lamports().checked_add(fee).ok_or(VotingError::MathError)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePull<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, space = 8 + mem::size_of::<Pull>())]
    pub pull: AccountLoader<'info, Pull>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateCandidate<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub pull: AccountLoader<'info, Pull>,
    #[account(init, payer = payer, space = 8 + mem::size_of::<Candidate>())]
    pub candidate: AccountLoader<'info, Candidate>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Voting<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub pull: AccountLoader<'info, Pull>,
    #[account(mut)]
    pub candidate: AccountLoader<'info, Candidate>,
    #[account(
        init,
        payer = payer,
        seeds = [b"voter", pull.key().as_ref(), payer.key().as_ref()],
        bump,
        space = 8 + mem::size_of::<Voter>()
    )]
    pub voter_tracker: AccountLoader<'info, Voter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePull<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut, close = creator)]
    pub pull: AccountLoader<'info, Pull>,
}

#[derive(Accounts)]
pub struct CloseCandidate<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut)]
    pub pull: AccountLoader<'info, Pull>,
    #[account(mut, close = creator)]
    pub candidate: AccountLoader<'info, Candidate>,
}

#[derive(Accounts)]
pub struct CloseVote<'info> {
    #[account(mut)]
    pub voter: SystemAccount<'info>,
    #[account(mut)]
    pub bot: Signer<'info>,
    #[account(mut, close = voter)]
    pub voter_tracker: AccountLoader<'info, Voter>,
}
