use solana_program::{pubkey::Pubkey, account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program_error::ProgramError, };
use crate::{sdk::{AccountInfoExt, system_program::SystemCpiExt}, state::candidate::Candidate};
use crate::state::voter::Voter;

pub struct VotingCtx<'a, 'info> { // probably i can just copy all accounts and remove 'a...
    pub payer: &'a AccountInfo<'info>,
    pub _pull: &'a AccountInfo<'info>,
    pub candidate: &'a AccountInfo<'info>,
    pub voter_pda: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,

    pub voter_seeds: [&'a [u8]; 3],
    pub voter_pda_bump: u8,
}

impl<'a, 'info> VotingCtx<'a, 'info> {
    pub fn parse(program_id: &Pubkey, accounts: &'a [AccountInfo<'info>]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();
        let voter = next_account_info(iter)?.assert_signer()?.assert_mut()?;
        let pull = next_account_info(iter)?.assert_owner(program_id)?;
        let candidate = next_account_info(iter)?.assert_owner(program_id)?.assert_mut()?;

        let voter_pda = next_account_info(iter)?.assert_mut()?.assert_empty()?; // todo: do i need 'empty' check here?
        let seeds = Voter::get_seeds(pull.key, voter.key);
        let voter_pda_bump = voter_pda.assert_pda(&seeds, program_id)?;

        Ok(Self {
            payer: voter,
            _pull: pull,
            candidate,
            voter_pda,
            system_program: next_account_info(iter)?.assert_system()?,

            voter_seeds: seeds,
            voter_pda_bump
        })
    }
}


pub fn voting<'a>(program_id: &Pubkey, accounts: &'a[AccountInfo<'a>], _data: &[u8]) -> ProgramResult {
    solana_program::msg!("ctx");
    let ctx = VotingCtx::parse(program_id, accounts)?;
    solana_program::msg!("ctx done");

    // todo: check time

    ctx.voter_pda.create_sdk_pda_cpi::<Voter>(
        ctx.payer,
        ctx.system_program,
        ctx.voter_seeds.as_ref(),
        ctx.voter_pda_bump,
        program_id
    )?;

    let mut candidate = ctx.candidate.load_mut::<Candidate>()?;
    candidate.number_of_votes += 1; // todo: handle

    let mut voter = ctx.voter_pda.load_mut::<Voter>()?;
    voter.voter_addr = *ctx.payer.key;
    voter.voted_for = *ctx.candidate.key;
    // todo: other

    Ok(())
}
