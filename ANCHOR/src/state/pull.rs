use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct PullMetadata {
    #[max_len(32)]
    pub title: String,
    #[max_len(128)]
    pub description: String,
    pub voting_start: i64,
    pub voting_end: i64,
}

#[account]
#[derive(InitSpace)]
pub struct Pull {
    pub creator: Pubkey,
    pub metadata: PullMetadata,
    // Voting price, which is frozen until the end of the voting period,
    // used to help prevent Sybil attacks
    pub vote_price: u64,
    pub candidate_count: u64,
    pub next_candidate_idx: u64,
    pub bump: u8,
}

impl Pull {
    pub fn get_pda_seed(title: &str) -> [&[u8]; 2] {
        [b"pull", title.as_bytes()]
    }

    pub fn find_pda(title: &str) -> (Pubkey, u8) {
        Pubkey::find_program_address(&Self::get_pda_seed(title), &crate::ID)
    }
}
