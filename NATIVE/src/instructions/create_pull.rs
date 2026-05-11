use crate::sdk::utils;
use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program::invoke, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};

use crate::{constants::{MAX_DESC_LEN, MAX_TITLE_LEN}, sdk::{AccountInfoExt, InstructionArgs}, state::pull::Pull};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct CreatePullArgs {
    pub(crate) title: [u8; MAX_TITLE_LEN],
    pub(crate) description: [u8; MAX_DESC_LEN],
    pub(crate) voting_start: i64,
    pub(crate) voting_end: i64,
    pub(crate) vote_price: u64,
}

impl InstructionArgs for CreatePullArgs {}

impl CreatePullArgs {
    #[cfg(not(target_os = "solana"))]
    pub fn new(title: &str, description: &str, voting_start: i64, voting_end: i64, vote_price: u64) -> Result<Self, ()> {
        let title = utils::string_to_bytes::<MAX_TITLE_LEN>(title)
            .ok_or(())?;  // todo! anyhow? thiserror?

        let description = utils::string_to_bytes::<MAX_DESC_LEN>(description)
            .ok_or(())?;  // todo! anyhow? thiserror?

        Ok(Self {
            title,
            description,
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
    use crate::sdk::off_chain::{ClientInstruction, IntoAccountMeta};

    pub struct CreatePullAccounts {
        pub payer: Pubkey,
        pub pull_key: Pubkey,
    }

    impl ClientInstruction for CreatePullArgs {
        type Accounts = CreatePullAccounts;
        const IX_TAG: u8 = crate::CREATE_PULL_IX;

        fn accounts_to_metas(accs: &Self::Accounts) -> Vec<solana_program::instruction::AccountMeta> {
            vec![
                accs.payer.mut_signer(),
                accs.pull_key.mut_signer(),
                solana_system_interface::program::ID.readonly(),
            ]
        }
    }
}
