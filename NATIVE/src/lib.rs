
pub mod sdk;
pub mod state;
pub mod instructions;
pub mod processor;
pub mod error;
pub mod constants;

pub const CREATE_PULL_IX: u8 = 1;
pub const CREATE_CANDIDATE_IX: u8 = 4;
pub const VOTE_IX: u8 = 8;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint {
    use crate::*;
    use solana_program::entrypoint;
    use solana_program::{
        account_info::AccountInfo,
        program_error::ProgramError,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    };

    entrypoint!(process_instruction);

    pub fn process_instruction<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {

        // todo: this code to macro or inline in sdk-func??
        if instruction_data.len() < 8 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (header, data) = instruction_data.split_at(8);
        let tag = header[0];
        // msg!("TAG: {}", tag);
        // msg!("data: {:?}", data);

        match tag {
            CREATE_PULL_IX => processor::create_pull::create_pull(program_id, accounts, data),
            CREATE_CANDIDATE_IX => processor::create_candidate::create_candidate(program_id, accounts, data),
            VOTE_IX => processor::voting::voting(program_id, accounts, data),
            _ => Err(ProgramError::InvalidInstructionData.into())
        }
    }
}
