use crate::sdk::utils;
use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program::invoke, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};

use crate::{constants::{MAX_DESC_LEN, MAX_TITLE_LEN}, sdk::{AccountInfoExt, InstructionArgs}, state::pull::Pull};

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
        const IX_TAG: u8 = crate::VOTE_IX;

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
