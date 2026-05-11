use bytemuck::{Pod, Zeroable};
use crate::sdk::utils;
use crate::{constants::MAX_CANDIDATE_NAME_LEN, sdk::{AccountInfoExt, InstructionArgs}, state::pull::Pull};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct CreateCandidateArgs {
    pub(crate) name: [u8; MAX_CANDIDATE_NAME_LEN],
}

impl InstructionArgs for CreateCandidateArgs {}

impl CreateCandidateArgs {
    #[cfg(not(target_os = "solana"))]
    pub fn new(name: &str) -> Result<Self, ()> {
        let name = utils::string_to_bytes::<MAX_CANDIDATE_NAME_LEN>(name)
            .ok_or(())?;  // todo! anyhow? thiserror?

        Ok(Self {
            name
        })
    }
}

// OFF-CHAIN LOGIC
#[cfg(not(target_os = "solana"))]
pub mod client {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use crate::sdk::off_chain::{ClientInstruction, IntoAccountMeta};

    pub struct CreateCandidateAccounts {
        pub payer: Pubkey,
        pub pull_key: Pubkey,
        pub candidate_key: Pubkey,
    }

    impl ClientInstruction for CreateCandidateArgs {
        type Accounts = CreateCandidateAccounts;
        const IX_TAG: u8 = crate::CREATE_CANDIDATE_IX;

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
