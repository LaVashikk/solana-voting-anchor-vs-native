use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program::invoke, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};
use solana_program::msg;
use solana_system_interface::instruction;

use crate::{constants::{MAX_DESC_LEN, MAX_TITLE_LEN}, instructions::create_pull::CreatePullArgs, sdk::InstructionArgs, state::pull::Pull};
use crate::sdk::{AccountInfoExt, AccountState, system_program::SystemCpiExt};

/// Context for creating new pull
/// Just like in Anchor, but handcrafted :p
pub struct CreatePullCtx<'a, 'info> { // probably i can just copy all accounts and remove 'a...
    pub payer: &'a AccountInfo<'info>,
    // Pull is a new keypair, not a PDA.
    // It must sign the transaction to prove ownership of the address
    pub pull: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> CreatePullCtx<'a, 'info> { // TODO: should i make it as trait?
    pub fn parse(_program_id: &Pubkey, accounts: &'a [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();
        Ok(Self {
            payer: next_account_info(iter)?.assert_signer()?.assert_mut()?,
            pull: next_account_info(iter)?.assert_signer()?.assert_empty()?.assert_mut()?,
            system_program: next_account_info(iter)?.assert_system()?,
        })
    }
}

pub fn create_pull<'a>(program_id: &Pubkey, accounts: &'a[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    let ctx = CreatePullCtx::parse(program_id, accounts)?;
    let args = CreatePullArgs::parse_from_bytes(data)?;

    // todo: create a macro for that task? Something like `create_with_init_and_get`, idk :p
    ctx.pull.create_sdk_account_cpi::<Pull>(ctx.payer, ctx.system_program, program_id)?;
    let mut pull_data = ctx.pull.load_mut::<Pull>()?;

    // uuugh, well... Dot notation, why not? T_T
    pull_data.creator = *ctx.payer.key;
    pull_data.title = args.title;
    pull_data.description = args.description;
    pull_data.title_len = args.title.iter().position(|&b| b == 0).unwrap_or(MAX_TITLE_LEN);
    pull_data.desc_len = args.description.iter().position(|&b| b == 0).unwrap_or(MAX_DESC_LEN);

    pull_data.voting_start = args.voting_start;
    pull_data.voting_end = args.voting_end;
    pull_data.vote_price = args.vote_price;
    pull_data.candidate_count = 0;
    pull_data.last_candidate = Pubkey::default();

    Ok(())
}
