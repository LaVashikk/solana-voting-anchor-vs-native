pub mod errors;
pub mod state;
pub mod instructions;

use anchor_lang::prelude::*;
pub use state::*;
use crate::instructions::*;

declare_id!("9AvUNHjxscdkiKQ8tUn12QCMXtcnbR9BVGq3ULNzFMRi");

#[program]
pub mod anchor_vote {
    use super::*;

    pub fn create_pull(ctx: Context<CreatePull>, metadata: PullMetadata, vote_price: u64) -> Result<()> {
        instructions::handle_create_pull(ctx, metadata, vote_price)
    }

    pub fn create_candidate(ctx: Context<CreateCandidate>, name: String) -> Result<()> {
        instructions::handle_create_candidate(ctx, name)
    }

    pub fn voting(ctx: Context<Voting>) -> Result<()> {
        instructions::handle_voting(ctx)
    }

    pub fn close_pull(ctx: Context<ClosePull>) -> Result<()> {
        instructions::handle_close_pull(ctx)
    }

    pub fn close_candidate(ctx: Context<CloseCandidate>) -> Result<()> {
        instructions::handle_close_candidate(ctx)
    }

    pub fn close_voting(ctx: Context<CloseVoting>) -> Result<()> {
        instructions::handle_close_voting(ctx)
    }
}
