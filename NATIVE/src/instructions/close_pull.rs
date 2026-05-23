use dummy_sdk::instruction::InstructionArgs;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ClosePullArgs;

impl InstructionArgs for ClosePullArgs {}

// OFF-CHAIN LOGIC
#[cfg(not(target_os = "solana"))]
pub mod client {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use dummy_sdk::client::{ClientInstruction, IntoAccountMeta};

    pub struct ClosePullAccounts {
        pub creator: Pubkey,
        pub pull: Pubkey,
    }

    impl ClientInstruction for ClosePullArgs {
        type Accounts = ClosePullAccounts;
        const IX_TAG: u64 = crate::CLOSE_PULL_IX;

        fn accounts_to_metas(accs: &Self::Accounts) -> Vec<solana_program::instruction::AccountMeta> {
            vec![
                accs.creator.mut_signer(),
                accs.pull.mut_owned(),
            ]
        }
    }
}
