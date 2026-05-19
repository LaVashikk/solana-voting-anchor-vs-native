use crate::sdk::{ForProgramExt, InitOwnedAccount, ParseAccountExt, ParseFrom, SignerAccount, SystemProgram, prelude::*};
use crate::{error::VotingError, instructions::create_pull::CreatePullArgs, state::pull::Pull};

/// Context for creating new pull
/// Just like in Anchor, but handcrafted :p
struct CreatePullCtx<'info> {
    pub payer: SignerAccount<'info>,
    // Pull is a new keypair, not a PDA.
    // It must sign the transaction to prove ownership of the address
    pub pull: InitOwnedAccount<'info, Pull>,
    pub system_program: SystemProgram<'info>,
}

impl<'info> CreatePullCtx<'info> { // TODO: should i make it as trait?
    pub fn parse(program_id: &'info Pubkey, accounts: &'info [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();
        Ok(Self {
            payer: next_account_info(iter)?.parse_into()?,
            pull: next_account_info(iter)?.require_signer()?.for_program(program_id).parse_into()?, // check and then convert
            // pull: next_account_info(iter)?.for_program(program_id).parse_into()?.require_signer()?, // not working
            // pull: InitOwnedAccount::parse_from( next_account_info(iter)?.for_program(program_id) )?.require_signer()?, // other option. idk.
            system_program: next_account_info(iter)?.parse_into()?,
        })
    }
}

pub fn create_pull<'a>(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    let ctx = CreatePullCtx::parse(program_id, accounts)?;
    let args = CreatePullArgs::parse_from_bytes(data)?;

    require!(args.voting_start < args.voting_end, VotingError::InvalidTimeRange);
    require!(args.voting_end > Clock::get()?.unix_timestamp, VotingError::VotingAlreadyEnded);

    // todo: create a macro for that task? Something like `create_with_init_and_get`. Or use something like AccountWrappers<T>
    ctx.pull.create_sdk_account_cpi::<Pull>(&ctx.payer, &ctx.system_program, program_id)?;

    let mut pull_data = ctx.pull.load_mut()?;
    pull_data.creator = *ctx.payer.key;
    pull_data.title = args.title;
    pull_data.description = args.description;

    pull_data.voting_start = args.voting_start;
    pull_data.voting_end = args.voting_end;
    pull_data.vote_price = args.vote_price;
    pull_data.candidate_count = 0;
    pull_data.last_candidate = PodOption::none();

    Ok(())
}
