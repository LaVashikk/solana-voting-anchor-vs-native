use std::{marker::PhantomData, ops::Deref};

use bytemuck::Pod;
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program::invoke, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};
use crate::{sdk::{AccountInfoExt, AccountState, system_program::SystemCpiExt}, state::candidate::Candidate};
use crate::state::voter::Voter;

// // IN SDK:
pub trait PdaSeeder: AccountState {
    type Context<'a, 'info> where 'info: 'a;

    fn with_seeds<'a, 'info, R, F>(ctx: &'a Self::Context<'a, 'info>, f: F) -> Result<R, ProgramError>
    where
        'info: 'a,
        F: FnOnce(&[&[u8]]) -> Result<R, ProgramError>;
}

// to macros?
impl PdaSeeder for Voter {
    type Context<'a, 'info> = VotingCtx<'a, 'info> where 'info: 'a;

    fn with_seeds<'a, 'info, R, F>(ctx: &'a Self::Context<'a, 'info>, f: F) -> Result<R, ProgramError>
    where
        'info: 'a,
        F: FnOnce(&[&[u8]]) -> Result<R, ProgramError>
    {
        f(&[b"voter", ctx._pull.key.as_ref(), ctx.voter.key.as_ref()])
    }
}
#[derive(Clone)]
pub struct ProgramAccountInfo<'a, 'info, T: AccountState + PdaSeeder> { // todo: use Lifetime Collapsing. like in anchor (idk why they do that. for DX?)
    pub account_info: &'a AccountInfo<'info>,
    _marker: PhantomData<T>,
}

impl<'a, 'info, T: AccountState + PdaSeeder> ProgramAccountInfo<'a, 'info, T> {
    pub fn create_pda(
        &self,
        ctx: &'a T::Context<'a, 'info>,
        program_id: &Pubkey,
        payer: &AccountInfo<'info>,
        system: &AccountInfo<'info>
    ) -> Result<u8, ProgramError> {
        // let seeds = Self::get_seeds(ctx);
        let space = T::SIZE;
        let disc = T::DISCRIMINATOR;
        // ..and other already here


        // PDA CHECKS!
        T::with_seeds(ctx, |seeds| {
            let (pda, bump) = Pubkey::find_program_address(&seeds, program_id);
            if pda != *self.key {
                return Err(ProgramError::InvalidSeeds)
            }

            // and here invoke_signer(
            // ....
            // ...)

            Ok(bump)
        })
    }
}

impl<'a, 'info, T: AccountState + PdaSeeder> Deref for ProgramAccountInfo<'a, 'info, T> {
    type Target = AccountInfo<'info>;
    fn deref(&self) -> &Self::Target {
        self.account_info
    }
}

impl<'a, 'info, T: AccountState + PdaSeeder> From<&'a AccountInfo<'info>> for ProgramAccountInfo<'a, 'info, T> {
    fn from(account_info: &'a AccountInfo<'info>) -> Self {
        Self {
            account_info,
            _marker: PhantomData,
        }
    }
}

pub struct VotingCtx<'a, 'info> {
    // pub args: VoterArgs // <- if needed
    pub voter: &'a AccountInfo<'info>,
    pub _pull: &'a AccountInfo<'info>,
    pub candidate: &'a AccountInfo<'info>,
    pub voter_pda: ProgramAccountInfo<'a, 'info, Voter>,
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> VotingCtx<'a, 'info> {
    pub fn parse(program_id: &Pubkey, accounts: &'a [AccountInfo<'info>], data: &[u8]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();

        Ok(Self {
            // args:: VoterArgs::parse(data),

            voter: next_account_info(iter)?.assert_owner(program_id)?.assert_mut()?,
            _pull: next_account_info(iter)?.assert_owner(program_id)?,
            candidate: next_account_info(iter)?.assert_empty()?.assert_signer()?.assert_mut()?,
            voter_pda: next_account_info(iter)?.assert_mut()?.assert_empty()?.into(), // TODO! нахуй проверки, тупо ProgramAccountInfo::TRY_FROM()? и все
            system_program: next_account_info(iter)?.assert_system()?,
        })
    }
}

pub fn voting<'a>(program_id: &Pubkey, accounts: &'a[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    let ctx = VotingCtx::parse(program_id, accounts, data)?;

    ctx.voter_pda.create_pda(&ctx, program_id, ctx.voter, ctx.system_program)?;

    let mut candidate = ctx.candidate.load_mut::<Candidate>()?;
    candidate.number_of_votes += 1; // todo: handle
    // todo: other

    todo!()
}
