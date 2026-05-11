use std::str::Utf8Error;
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use crate::{constants::{MAX_DESC_LEN, MAX_TITLE_LEN}, sdk::{AccountState, Discriminator}};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Pull {
    pub creator: Pubkey,

    pub title: [u8; MAX_TITLE_LEN],
    pub title_len: usize,
    pub description: [u8; MAX_DESC_LEN],
    pub desc_len: usize,

    pub voting_start: i64,
    pub voting_end: i64,

    // Voting price, which is frozen until the end of the voting period,
    // used to help prevent Sybil attacks
    pub vote_price: u64,

    pub candidate_count: u64,
    pub last_candidate: Pubkey // todo: or make something like `PodOption`
}

impl Discriminator for Pull {
    const DISCRIMINATOR: u8 = 1; // todo: use const value
}

impl Pull {
    pub fn get_title(&self) -> Result<&str, Utf8Error> {
        let len = (self.title_len).min(MAX_TITLE_LEN);
        std::str::from_utf8(&self.title[..len])
    }

    pub fn get_description(&self) -> Result<&str, Utf8Error> {
        let len = (self.desc_len).min(MAX_DESC_LEN);
        std::str::from_utf8(&self.description[..len])
    }

    pub fn get_last_candidate(&self) -> Option<&Pubkey> {
        if self.last_candidate != Pubkey::default() {
            return Some(&self.last_candidate)
        }
        None
    }
}
