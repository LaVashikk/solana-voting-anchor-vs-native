use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use crate::{constants::MAX_CANDIDATE_NAME_LEN, sdk::{Discriminator, pod_types::string::FixedString}};
use crate::sdk::pod_types::option::PodOption;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Candidate {
    pub pull: Pubkey,
    pub name: FixedString<MAX_CANDIDATE_NAME_LEN>,

    pub number_of_votes: u64,
    pub last_candidate: PodOption<Pubkey>,
}

impl Discriminator for Candidate {
    const DISCRIMINATOR: u8 = 2; // todo: use const value
}
