use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use dummy_sdk::prelude::*;
use crate::constants::MAX_CANDIDATE_NAME_LEN;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Candidate {
    pub pull: Pubkey,
    pub name: FixedString<MAX_CANDIDATE_NAME_LEN>,

    pub number_of_votes: u64,
    pub prev_candidate: PodOption<Pubkey>,
}

impl Discriminator for Candidate {
    const DISCRIMINATOR: u8 = crate::constants::CANDIDATE_DISC;
}
