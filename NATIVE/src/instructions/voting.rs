use bytemuck::{Pod, Zeroable};
use crate::sdk::InstructionArgs;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct VotingArgs;

impl InstructionArgs for VotingArgs {}

// OFF-CHAIN LOGIC
#[cfg(not(target_os = "solana"))]
pub mod client {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use crate::sdk::off_chain::{ClientInstruction, IntoAccountMeta};

    pub struct VotingAccounts {
        pub voter: Pubkey,
        pub pull: Pubkey,
        pub candidate: Pubkey,
        pub voter_pda: Pubkey,
    }

    impl ClientInstruction for VotingArgs {
        type Accounts = VotingAccounts;
        const IX_TAG: u64 = crate::CREATE_VOTE_IX;

        fn accounts_to_metas(accs: &Self::Accounts) -> Vec<solana_program::instruction::AccountMeta> {
            vec![
                accs.voter.mut_signer(),
                accs.pull.readonly(),
                accs.candidate.mut_owned(),
                accs.voter_pda.mut_owned(),
                solana_system_interface::program::ID.readonly(),
            ]
        }
    }
}
