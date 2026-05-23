use dummy_sdk::prelude::*;
use bytemuck::{Pod, Zeroable};
use crate::constants::{MAX_DESC_LEN, MAX_TITLE_LEN};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct CreatePullArgs {
    pub(crate) title: FixedString<MAX_TITLE_LEN>,
    pub(crate) description: FixedString<MAX_DESC_LEN>,
    pub(crate) voting_start: i64,
    pub(crate) voting_end: i64,
    pub(crate) vote_price: u64,
}

impl InstructionArgs for CreatePullArgs {}

impl CreatePullArgs {
    #[cfg(not(target_os = "solana"))]
    pub fn new(title: &str, description: &str, voting_start: i64, voting_end: i64, vote_price: u64) -> anyhow::Result<Self> {
        Ok(Self {
            title: FixedString::try_new(title)?,
            description: FixedString::try_new(description)?,
            voting_start,
            voting_end,
            vote_price,
        })
    }
}

// OFF-CHAIN LOGIC
#[cfg(not(target_os = "solana"))]
pub mod client {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use dummy_sdk::client::{ClientInstruction, IntoAccountMeta};

    pub struct CreatePullAccounts {
        pub payer: Pubkey,
        pub pull_key: Pubkey,
    }

    impl ClientInstruction for CreatePullArgs {
        type Accounts = CreatePullAccounts;
        const IX_TAG: u64 = crate::CREATE_PULL_IX;

        fn accounts_to_metas(accs: &Self::Accounts) -> Vec<solana_program::instruction::AccountMeta> {
            vec![
                accs.payer.mut_signer(),
                accs.pull_key.mut_signer(),
                solana_system_interface::program::ID.readonly(),
            ]
        }
    }
}
