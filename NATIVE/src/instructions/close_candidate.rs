use dummy_sdk::instruction::InstructionArgs;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct CloseCandidateArgs;

impl InstructionArgs for CloseCandidateArgs {}

// OFF-CHAIN LOGIC
#[cfg(not(target_os = "solana"))]
pub mod client {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use dummy_sdk::client::{ClientInstruction, IntoAccountMeta};

    pub struct CloseCandidateAccounts {
        pub creator: Pubkey,
        pub pull: Pubkey,
        pub candidate: Pubkey,
    }

    impl ClientInstruction for CloseCandidateArgs {
        type Accounts = CloseCandidateAccounts;
        const IX_TAG: u64 = crate::CLOSE_CANDIDATE_IX;

        fn accounts_to_metas(accs: &Self::Accounts) -> Vec<solana_program::instruction::AccountMeta> {
            vec![
                accs.creator.mut_signer(),
                accs.pull.mut_owned(),
                accs.candidate.mut_owned(),
            ]
        }
    }
}
