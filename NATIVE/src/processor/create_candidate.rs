use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program_error::ProgramError};
use solana_program::msg;
use solana_program::pubkey::Pubkey;
use solana_system_interface::instruction;

use crate::{constants::MAX_CANDIDATE_NAME_LEN, instructions::create_candidate::CreateCandidateArgs, sdk::InstructionArgs, state::{candidate::Candidate, pull::Pull}};
use crate::sdk::{AccountInfoExt, AccountState, system_program::SystemCpiExt};

/// Context for creating new candidate
/// Just like in Anchor, but handcrafted :p
pub struct CreateCandidateCtx<'a, 'info> { // probably i can just copy all accounts and remove 'a...
    pub payer: &'a AccountInfo<'info>,
    pub pull: &'a AccountInfo<'info>,
    pub candidate: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> CreateCandidateCtx<'a, 'info> {
    pub fn parse(program_id: &Pubkey, accounts: &'a [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();
        Ok(Self {
            payer: next_account_info(iter)?.assert_signer()?.assert_mut()?,
            pull: next_account_info(iter)?.assert_owner(program_id)?.assert_mut()?,
            candidate: next_account_info(iter)?.assert_empty()?.assert_signer()?.assert_mut()?,
            system_program: next_account_info(iter)?.assert_system()?,
        })
    }
}

pub fn create_candidate<'a>(program_id: &Pubkey, accounts: &'a[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    let ctx = CreateCandidateCtx::parse(program_id, accounts)?;
    let args = CreateCandidateArgs::parse_from_bytes(data)?;

    ctx.candidate.create_sdk_account_cpi::<Candidate>(ctx.payer, ctx.system_program, program_id)?;

    let mut candidate_data = ctx.candidate.load_mut::<Candidate>()?;
    candidate_data.pull = *ctx.pull.key;
    candidate_data.name = args.name;
    candidate_data.name_len = args.name.iter().position(|&b| b == 0).unwrap_or(MAX_CANDIDATE_NAME_LEN);

    let mut pull_data = ctx.pull.load_mut::<Pull>()?;
    pull_data.candidate_count = pull_data.candidate_count.saturating_add(1);

    Ok(())
}
