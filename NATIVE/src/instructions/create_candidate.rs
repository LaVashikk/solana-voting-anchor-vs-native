use dummy_sdk::prelude::*;
use bytemuck::{Pod, Zeroable};
use crate::constants::MAX_CANDIDATE_NAME_LEN;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct CreateCandidateArgs {
    pub(crate) name: FixedString<MAX_CANDIDATE_NAME_LEN>,
}

impl InstructionArgs for CreateCandidateArgs {}
impl CreateCandidateArgs {
    #[cfg(not(target_os = "solana"))]
    pub fn new(name: &str) -> anyhow::Result<Self> {
        Ok(Self {
            name: FixedString::try_new(name)?
        })
    }
}

// OFF-CHAIN LOGIC
#[cfg(not(target_os = "solana"))]
pub mod client {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use dummy_sdk::client::{ClientInstruction, IntoAccountMeta};

    pub struct CreateCandidateAccounts {
        pub payer: Pubkey,
        pub pull_key: Pubkey,
        pub candidate_key: Pubkey,
    }

    impl ClientInstruction for CreateCandidateArgs {
        type Accounts = CreateCandidateAccounts;
        const IX_TAG: u64 = crate::CREATE_CANDIDATE_IX;

        fn accounts_to_metas(accs: &Self::Accounts) -> Vec<solana_program::instruction::AccountMeta> {
            vec![
                accs.payer.mut_signer(),
                accs.pull_key.mut_owned(),
                accs.candidate_key.mut_signer(), // !!TODO: if it's wrong - super annoing weird errors: `panicked at /home/lavashik/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/solana-transaction-3.1.0/src/lib.rs:719:13:`
                solana_system_interface::program::ID.readonly(),
            ]
        }
    }
}
