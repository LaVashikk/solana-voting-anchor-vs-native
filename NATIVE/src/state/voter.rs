use dummy_sdk::accounts::Discriminator;
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Voter {
    pub voter_addr: Pubkey,
    pub voted_for: Pubkey,
    pub voting_end: i64,
    // pub bump: u8, // todo: uuugh. for WHAT? im not checking PDA's, only discriptor and is_owned to program_id
    // _padding: [u8; 7]
}

impl Discriminator for Voter {
    const DISCRIMINATOR: u8 = crate::constants::VOTE_DISC;
}

impl Voter {
    #[inline]
    pub fn get_seeds<'a>(pull_key: &'a Pubkey, voter_key: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            b"voter",
            pull_key.as_ref(),
            voter_key.as_ref()
        ]
    }
}
