use dummy_sdk::instruction::InstructionArgs;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct CloseVoteArgs;

impl InstructionArgs for CloseVoteArgs {}

// OFF-CHAIN LOGIC
#[cfg(not(target_os = "solana"))]
pub mod client {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use dummy_sdk::client::{ClientInstruction, IntoAccountMeta};

    pub struct CloseVoteAccounts {
        pub voter: Pubkey,
        pub bot: Pubkey,
        pub voter_tracker: Pubkey,
    }

    impl ClientInstruction for CloseVoteArgs {
        type Accounts = CloseVoteAccounts;
        const IX_TAG: u64 = crate::CLOSE_VOTE_IX;

        fn accounts_to_metas(accs: &Self::Accounts) -> Vec<solana_program::instruction::AccountMeta> {
            vec![
                accs.voter.mut_owned(),
                accs.bot.mut_signer(),
                accs.voter_tracker.mut_owned(),
            ]
        }
    }
}
