use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use crate::{constants::MAX_CANDIDATE_NAME_LEN, sdk::{Discriminator, pod_types::string::FixedString}};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Candidate {
    pub pull: Pubkey,
    pub name: FixedString<MAX_CANDIDATE_NAME_LEN>,

    pub number_of_votes: u64,
    pub last_candidate: Pubkey,
}

impl Discriminator for Candidate {
    const DISCRIMINATOR: u8 = 2; // todo: use const value
}

impl Candidate {
    pub fn get_last_candidate(&self) -> Option<&Pubkey> {
        if self.last_candidate != Pubkey::default() {
            return Some(&self.last_candidate)
        }
        None
    }

    pub fn get_all_candidates<'a>(&'a self) -> Vec<&'a Pubkey> {
        todo!()
    }
}
