use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use crate::{constants::{MAX_DESC_LEN, MAX_TITLE_LEN}, sdk::{Discriminator, pod_types::{option::PodOption, string::FixedString}}};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Pull {
    pub creator: Pubkey,

    pub title: FixedString<MAX_TITLE_LEN>,
    pub description: FixedString<MAX_DESC_LEN>,

    pub voting_start: i64,
    pub voting_end: i64,

    // Voting price, which is frozen until the end of the voting period,
    // used to help prevent Sybil attacks
    pub vote_price: u64,

    pub candidate_count: u64,
    pub last_candidate: PodOption<Pubkey>,
}

impl Discriminator for Pull {
    const DISCRIMINATOR: u8 = 1; // todo: use const value
}

impl Pull {
    pub fn get_all_candidates<'a>(&'a self) -> Vec<&'a Pubkey> {
        // no idea how without runtime/rpc
        todo!()
    }
}
