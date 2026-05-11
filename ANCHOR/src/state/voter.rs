use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Voter {
    pub voter_addr: Pubkey,
    pub voted_for: Pubkey,
    pub voting_end: i64,
}

impl Voter {
    pub fn get_pda_seed<'a>(pull: &'a Pubkey, voter: &'a Pubkey) -> [&'a [u8]; 3] {
        [b"voter", pull.as_ref(), voter.as_ref()]
    }

    pub fn find_pda(pull: &Pubkey, voter: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&Self::get_pda_seed(pull, voter), &crate::ID)
    }
}
