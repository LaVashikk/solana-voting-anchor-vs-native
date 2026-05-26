use anchor_lang::prelude::*;
use dummy_sdk::prelude::*;

pub const MAX_TITLE_LEN: usize = 32;
pub const MAX_DESC_LEN: usize = 32;
pub const MAX_CANDIDATE_NAME_LEN: usize = 32;

#[account(zero_copy)]
#[repr(C)]
pub struct Pull {
    pub creator: Pubkey,
    pub title: FixedString<MAX_TITLE_LEN>,
    pub description: FixedString<MAX_DESC_LEN>,
    pub voting_start: i64,
    pub voting_end: i64,
    pub vote_price: u64,
    pub candidate_count: u64,
    pub last_candidate: PodOption<Pubkey>,
    pub next_candidate_idx: u64,
}

#[account(zero_copy)]
#[repr(C)]
pub struct Candidate {
    pub pull: Pubkey,
    pub name: FixedString<MAX_CANDIDATE_NAME_LEN>,
    pub number_of_votes: u64,
    pub prev_candidate: PodOption<Pubkey>,
}

#[account(zero_copy)]
#[repr(C)]
pub struct Voter {
    pub voter_addr: Pubkey,
    pub voted_for: Pubkey,
    pub voting_end: i64,
}

impl Voter {
    pub fn get_seeds<'a>(pull_key: &'a Pubkey, voter_key: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            b"voter",
            pull_key.as_ref(),
            voter_key.as_ref()
        ]
    }
}
