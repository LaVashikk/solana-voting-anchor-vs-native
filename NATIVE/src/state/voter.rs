use std::str::Utf8Error;
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;

use crate::sdk::AccountInfoExt;
use crate::sdk::Discriminator;
use crate::{constants::{MAX_DESC_LEN, MAX_TITLE_LEN}, sdk::AccountState};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Voter {
    pub voter_addr: Pubkey,
    pub voted_for: Pubkey,
    pub voting_end: i64,
    // pub bump: u8, // todo: uuugh. for WHAT? im not checking PDA's, only discriptor and is_owned to program_id
    // _padding: [u8; 7]
}

impl Discriminator for Voter {
    const DISCRIMINATOR: u8 = 4; // todo: use const value
}


impl Voter {
    // Okay, thinking about it - this is actually a quite convenient and flexible approach!
    // Can we come up with any abstractions here? Yes, there are a few different ideas that would be convenient in combination with traits and macros
    // For example:
    // ```
    // pub trait PdaState<'a, const SEED_COUNT: usize> {
    //     type SeedArgs;
    //     fn build_seeds(args: &'a Self::SeedArgs) -> [&'a [u8]; SEED_COUNT];
    // }
    // ```
    // But uuugh.. I'd have to create '...SeedArgs' structs for each PDA state, buehhh. AAnd it's kinda overengineering

    #[inline]
    pub fn get_seeds<'a>(pull_key: &'a Pubkey, voter_key: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            b"voter",
            pull_key.as_ref(),
            voter_key.as_ref()
        ]
    }
}

// 'with_seeds' will be so epic-cool! Something like this one:
// Voter::with_seeds(ctx.pull.key, ctx.payer.key, |seeds| {
//     // Aaaaaaaaaand for example something like this?
//     ctx.voter_tracker.checked_create_sdk_pda_cpi::<Voter>(
//         ctx.payer,
//         ctx.system_program,
//         program_id,
//         seeds
//     )
// })?;
// Not bad, huh??? But I have no idea how to do it.. Macro?
