
pub mod sdk;
pub mod state;
pub mod instructions;
pub mod processor;
pub mod error;
pub mod constants;

pub const CREATE_PULL_IX: u64       = 1001;
pub const CREATE_CANDIDATE_IX: u64  = 1002;
pub const CREATE_VOTE_IX: u64       = 1003;
pub const CLOSE_PULL_IX: u64        = 2001;
pub const CLOSE_CANDIDATE_IX: u64   = 2002;
pub const CLOSE_VOTE_IX: u64        = 2003;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint {
    use crate::*;
    use sdk::prelude::*;

    solana_program::entrypoint!(process_instruction);

    pub fn process_instruction<'a>(
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {

        route_instructions! {
            program_id, accounts, instruction_data;

            CREATE_PULL_IX      => processor::create_pull::create_pull,
            CREATE_CANDIDATE_IX => processor::create_candidate::create_candidate,
            CREATE_VOTE_IX      => processor::create_vote::voting,

            // CLOSE_PULL_IX =>
            // CLOSE_CANDIDATE_IX =>
            // CLOSE_VOTE_IX =>
        }

    }
}
