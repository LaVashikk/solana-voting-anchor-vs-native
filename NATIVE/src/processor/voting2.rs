use std::{marker::PhantomData, ops::Deref};

use bytemuck::Pod;
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program::invoke, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};
use crate::{sdk::{AccountInfoExt, AccountState, system_program::SystemCpiExt}, state::candidate::Candidate};
use crate::state::voter::Voter;

// IN SDK:
trait ToSeed: AccountState {
    type Context<'a, 'info>;

    fn with_seeds<'a, 'info, R, F>(ctx: &'a Self::Context<'a, 'info>, f: F) -> Result<R, ProgramError>
    where
        F: FnOnce(&[&[u8]]) -> Result<R, ProgramError>;

}

// trait ProgramAccout: ToSeed { // this trait takes 1 generic argument but 0 generic arguments were supplied (???)
//     fn create<'a>(ctx: &'a Self::Context, program_id: &Pubkey) -> ProgramResult;
// }

// // impl ProgramAccout for AccountInfo where NOTHING BLYAT { // todo: беда. ToSeed реализовывает СТРУКТУРА, например Voter. А AccountInfo лишь данные из on-chain, которые В ТЕОРИИ мы интерпретируем как Voter
// // и struct Voter не может быть оберткой над AccountInfo, ведь мы должны модифицировать данные этой структуры, делая каст raw bytes в эту структуру. А делать 2 структуры - кринж наверное
// fn create(&self, ctx: &Self::Context, program_id: &Pubkey) -> ProgramResult {
//         let seeds = Self::get_seeds(ctx);
//         let space = Self::SIZE;
//         let disc = Self::DISCRIMINATOR;
//         // ..and other already here

//         // PDA CHECKS!
//         let (pda, bump) = Pubkey::find_program_address(&seeds, program_id);
//         if pda != self.pu // todo: FUCK!! self its NOT AccountInfo!! fuck!

//         // and here invoke_signer(
//         // ....
//         // ...)

//         Ok(())
//     }
// }


// ------
#[derive(Clone)]
#[repr(C)]
pub struct ProgramAccountInfo<'info, T: AccountState> { //  + ToSeed - сука, и этому хуеблядке нужен N. И как быть?? мне нужен get_seeds, НО МНЕ НЕ НУЖНО указывать количество сидов в ProgramAccountInfo. Эти данные указывается в структуру T
    pub account_info: &'info AccountInfo<'info>,
    _marker: PhantomData<T>, // todo: i can use it with repr C? // todo 2: и как мне это поможет?? по задумке - мне это надо для T::SIZE T::DISCRIMINATOR T::get_seeds!
} // okay... what's next?

impl ProgramAccountInfo {
    // pub fn create(ctx: &'a T::Context, program_id: &Pubkey) ... // ага, и для T::Context мне нужен ToSeeds!!
    // todo: заебись, чтобы в ProgramAccountInfo, который является частью Ctx получить seeds, надо передать в метод ProgramAccountInfo, который является частью Ctx сам Ctx. Ага блять, удачи
}

impl<'info, T: AccountState> Deref for ProgramAccountInfo<'info, T> {
    type Target = AccountInfo<'info>;
    fn deref(&self) -> &Self::Target {
        self.account_info
    }
}

pub struct VotingCtx<'a, 'info> { // probably i can just copy all accounts and remove 'a...
    // pub args: VoterArgs // <- if needed
    pub voter: &'a AccountInfo<'info>,
    pub _pull: &'a AccountInfo<'info>,
    pub candidate: &'a AccountInfo<'info>,
    pub voter_pda: &'a ProgramAccountInfo<'info, Voter>, // Окей идея 2: мы делаем обертку ProgramAccountInfo, которая реализует DEREF to orig!
    pub system_program: &'a AccountInfo<'info>,
}

impl ToSeed<3> for Voter {
    type Context = VotingCtx<'_, '_>;  // FUCK!!
    // missing lifetime specifier
    // `'_` cannot be used here
    // I DONT CARE UUUUUUUUGHHHHHHHHHH JUST GIMME USE THAT FOR MY METHOD!

    fn get_seeds<'a>(ctx: &'a Self::Context) -> [&'a [u8]; 3] {
        // and i can use args here also!
        [
            b"voter",
            ctx._pull.key.as_ref(),
            ctx.voter_pda.key.as_ref()
        ]
    }
}

impl<'a, 'info> VotingCtx<'a, 'info> {
    pub fn parse(program_id: &Pubkey, accounts: &'a [AccountInfo<'info>], data: &[u8]) -> Result<Self, ProgramError> {
        let iter = &mut accounts.iter();

        Ok(Self {
            // args:: VoterArgs::parse(data),

            voter: next_account_info(iter)?.assert_owner(program_id)?.assert_mut()?,
            _pull: next_account_info(iter)?.assert_owner(program_id)?,
            candidate: next_account_info(iter)?.assert_empty()?.assert_signer()?.assert_mut()?,
            voter_pda: next_account_info(iter)?.assert_mut()?.assert_empty()?, // THERE IS NO PDA-CHECKS!!
            system_program: next_account_info(iter)?.assert_system()?,
        })
    }
}

pub fn voting<'a>(program_id: &Pubkey, accounts: &'a[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    let ctx = VotingCtx::parse(program_id, accounts, data)?;

    // todo: check time

    ctx.voter_pda.create_sdk_pda_cpi::<Voter>(
        ctx.voter,
        ctx.system_program,
        ctx.voter_seeds.as_ref(),
        ctx.voter_pda_bump,
        program_id
    )?;

    let mut candidate = ctx.candidate.load_mut::<Candidate>()?;
    candidate.number_of_votes += 1; // todo: handle
    // todo: other

    todo!()
}
