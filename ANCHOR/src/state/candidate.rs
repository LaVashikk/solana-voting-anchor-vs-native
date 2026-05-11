use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Candidate {
    pub pull_key: Pubkey,
    #[max_len(32)]
    pub name: String,
    pub bump: u8,
    pub number_of_votes: u64,
}

impl Candidate {
    pub fn get_pda_seed<'a>(pull: &'a Pubkey, index_bytes: &'a [u8]) -> [&'a [u8]; 3] {
        [b"candidate", pull.as_ref(), index_bytes]
    }

    pub fn find_pda<'a>(pull: &'a Pubkey, index_bytes: &'a [u8]) -> (Pubkey, u8) {
        Pubkey::find_program_address(&Self::get_pda_seed(pull, index_bytes), &crate::ID)
    }
}
