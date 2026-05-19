use std::{cell::{Ref, RefMut}, marker::PhantomData, ops::Deref};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use bytemuck::{AnyBitPattern, Pod};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;

pub mod pod_types;
pub mod error;
pub mod cu_measuring;


// todo: name sounds like shit, make better
pub struct ProgramCtx<'info> {
    pub info: &'info solana_program::account_info::AccountInfo<'info>,
    pub program_id: &'info Pubkey,
}
pub trait ForProgramExt<'info> {
    fn for_program(self, program_id: &'info Pubkey) -> ProgramCtx<'info>;
}

impl<'info> ForProgramExt<'info> for &'info AccountInfo<'info> {
    #[inline(always)]
    fn for_program(self, program_id: &'info Pubkey) -> ProgramCtx<'info> {
        ProgramCtx { info: self, program_id }
    }
}

// AWESOME SAUcE
#[diagnostic::on_unimplemented(
    message = "Cannot parse account type `{Self}` from context `{Ctx}`",
    label = "missing `program_id` for owner validation",
    note = "Owned state accounts require a program ID to safely verify account ownership.\nCall `.for_program(program_id)` to wrap the account info before parsing.\nExample: `next_account_info(iter)?.for_program(program_id).parse_into()?`"
)]
pub trait ParseFrom<'info, Ctx> {
    fn parse_from(ctx: Ctx) -> Result<Self, ProgramError>
    where
        Self: Sized;
}

pub trait ParseAccountExt<'info> {
    #[inline(always)]
    fn parse_into<T>(self) -> Result<T, ProgramError>
    where
        T: ParseFrom<'info, Self>,
        Self: Sized,
    {
        T::parse_from(self)
    }
}
impl<'info, C> ParseAccountExt<'info> for C {}


#[macro_export]
macro_rules! declare_state_wrapper {
    (
        $wrapper_name:ident,
        |$info:ident, $program_id:ident| $validation:expr
    ) => {
        #[derive(Clone)]
        pub struct $wrapper_name<'info, T: AccountState> {  // todo: $crate for state
            pub info: &'info ::solana_program::account_info::AccountInfo<'info>,
            _marker: ::std::marker::PhantomData<T>,
        }

        impl<'info, T: AccountState> Deref for $wrapper_name<'info, T> {
            type Target = ::solana_program::account_info::AccountInfo<'info>;
            fn deref(&self) -> &Self::Target {
                self.info
            }
        }

        impl<'info, T: AccountState> ParseFrom<'info, ProgramCtx<'info>> for $wrapper_name<'info, T> {
            fn parse_from(ctx: ProgramCtx<'info>) -> Result<Self, ProgramError> {
                let $info = ctx.info;
                let $program_id = ctx.program_id;
                $validation;

                Ok(Self {
                    info: ctx.info,
                    _marker: std::marker::PhantomData,
                })
            }
        }

        // impl<'info, T: AccountState> TryFrom<$crate::sdk::ProgramCtx<'info>> for $wrapper_name<'info, T> {
        //     type Error = ProgramError;

        //     fn try_from(ctx: $crate::sdk::ProgramCtx<'info>) -> Result<Self, Self::Error> {
        //         let $info = ctx.info;
        //         let $program_id = ctx.program_id;
        //         $validation;

        //         Ok(Self {
        //             $info,
        //             _marker: ::std::marker::PhantomData,
        //         })
        //     }
        // }

        impl<'info, T: AccountState> $wrapper_name<'info, T> {
            pub fn load(&self) -> Result<Ref<'_, T>, ProgramError> {
                self.info.load::<T>()
            }

            pub fn load_mut(&self) -> Result<RefMut<'_, T>, ProgramError> {
                self.info.load_mut::<T>()
            }

            pub fn with_mut<F>(&self, f: F) -> Result<(), ProgramError> where F: FnOnce(&mut T) {
                self.info.with_mut::<T, _>(f)
            }
        }
    };
}

#[macro_export]
macro_rules! declare_account_wrapper {
    (
        $wrapper_name:ident,
        |$info:ident| $validation:expr
    ) => {
        #[derive(Clone)]
        pub struct $wrapper_name<'info> {  // todo: $crate for state
            pub info: &'info ::solana_program::account_info::AccountInfo<'info>,
        }

        impl<'info> Deref for $wrapper_name<'info> {
            type Target = ::solana_program::account_info::AccountInfo<'info>;
            fn deref(&self) -> &Self::Target {
                self.info
            }
        }

        // todo: fucking bullshit?
        impl<'info> ParseFrom<'info, &'info ::solana_program::account_info::AccountInfo<'info>> for $wrapper_name<'info> {
            fn parse_from($info: &'info ::solana_program::account_info::AccountInfo<'info>) -> Result<Self, ProgramError> {
                $validation;

                Ok(Self {
                    info: $info,
                })
            }
        }

        // impl<'info> TryFrom<&'info ::solana_program::account_info::AccountInfo<'info>> for $wrapper_name<'info> {
        //     type Error = ProgramError;

        //     fn try_from($info: &'info ::solana_program::account_info::AccountInfo<'info>) -> Result<Self, Self::Error> {
        //         $validation;

        //         Ok(Self {
        //             $info,
        //         })
        //     }
        // }
    };
}

// TODO:
// Idea for implementing Borsh support:
// 1. feature-flag
// 2. Wrapper type! Like ProgramAccount. Something like BorshAccount, for example
// 3. Unlike Anchor, DO NOT STORE the deserialized T!
//      BorshAccount implements deref to info, as well as get / get_mut methods. Each call deserializes AGANE. Although a cache could be added, it's overengineering imo
// 4. get_mut returns not just data, but ANOTHER wrapper, like BorshMutData, which implements drop. Drop serializes data back... nahhh, drop is kidna bad design! `save`? `commit`? oh, or maybe use closure?!
// Overall - that's probably it?
//
// TODO: study BORSH specifics. What makes it so special, why is it better than, say, bincode? we could cleverly record struct offsets, for example:
// implement a zero-cost method that takes a str (for example) and uses a match like:
// ```
//      // simply numbering the order of the structure fields. a derive macro can do this automatically
// .. fn get_order(field_name: &str) -> usize {
//      match field_name {
//          "manually writing the name":  1,
//          "field 2":  2,
//      }
// }
// ```
// Then the serializer will write RAW usizes in the first bytes! For example, we can allocate 8 bytes (idk why so many) for the offset, and then we get get_order * 8, which contains info on where the data starts and ends in bytes, and we deserialize precisely
// yeah, good luck to me "deserializing precisely", but some workarounds can be invented lol. This doesn't seem to exist in borsh/anchor (upd. anchor already has such a "workaround" - LazyAccount. they beat me to it. and I'm scared to study that code)
// ~~oh god, I just need to find a job, ahah ;p~~


#[cfg(not(target_os = "solana"))]
pub mod off_chain;

pub mod system_program;
pub mod utils;
pub mod prelude;

/// Discriminator size
const DISC_SIZE: usize = 8;

pub trait Discriminator: Sized {
    const DISCRIMINATOR: u8;
    const RAW_SIZE: usize = std::mem::size_of::<Self>();
    const SIZE: usize = std::mem::size_of::<Self>() + DISC_SIZE;
}

pub trait AccountState: Pod + Discriminator {
    fn try_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        if data[0] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData); // todo: use custom SDK error
        }

        Ok(bytemuck::from_bytes(&data[DISC_SIZE..Self::SIZE]))
    }

    fn try_from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        if data[0] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(bytemuck::from_bytes_mut(&mut data[DISC_SIZE..Self::SIZE]))
    }
}

// todo: make it better?
impl<T> AccountState for T where T: Pod + Discriminator {}

// TODO: made it INLINE?
pub trait AccountValidationExt: Sized {
    fn require<F: Fn(&Self) -> bool>(self, condition: F) -> Result<Self, ProgramError>;
    fn require_signer(self) -> Result<Self, ProgramError>;
    fn require_mut(self) -> Result<Self, ProgramError>;
    fn require_owner(self, program_id: &Pubkey) -> Result<Self, ProgramError>;
    fn require_adress(self, adress: &Pubkey) -> Result<Self, ProgramError>;
    fn require_system(self) -> Result<Self, ProgramError>;
    fn require_empty(self) -> Result<Self, ProgramError>;
    fn require_bumped_pda(self, program_id: &Pubkey, seeds: &[&[u8]], bump: u8) -> Result<Self, ProgramError>;

    // fn find_and_verify_pda(&self, seeds: &[&[u8]], program_id: &Pubkey) -> Result<u8, ProgramError>;
}

pub trait PdaExt { // fuck me, todo: rename, restuct, idk.
    fn find_and_verify_pda(&self, seeds: &[&[u8]], program_id: &Pubkey) -> Result<u8, ProgramError>;
}

pub trait AccountBytemuckExt {
    fn load<T: AccountState>(&self) -> Result<Ref<'_, T>, ProgramError>;
    fn load_mut<T: AccountState>(&self) -> Result<RefMut<'_, T>, ProgramError>;
    fn with_mut<T: AccountState, F>(&self, f: F) -> Result<(), ProgramError> where F: FnOnce(&mut T);
}

// pub struct OwnedState<'a, T: AccountState> {
//     pub info: &'a AccountInfo<'a>,
//     _marker: std::marker::PhantomData<T>,
// }

// // deref for OwnedState here!

// impl<'a, T: AccountState> OwnedState<'a, T> {
//     pub fn load(&'a self) -> Result<Ref<'a, T>, ProgramError> {
//         self.info.load::<T>()
//     }
// }

declare_account_wrapper! {
    SystemProgram,
    |info| info.require_system()?
}

declare_account_wrapper! {
    SignerAccount,
    |info| info.require_signer()?
}

declare_account_wrapper! {
    SignerAccountMut,
    |info| info.require_signer()?.require_mut()?
}

declare_state_wrapper! {
    OwnedAccount,
    |info, pid| info.require_owner(pid)?
}

declare_state_wrapper! {
    OwnedAccountMut,
    |info, pid| info.require_owner(pid)?.require_mut()?
}

declare_state_wrapper! {
    InitOwnedAccount,
    |info, _pid| info.require_empty()?.require_mut()?.require_owner(&SYSTEM_PROGRAM_ID)
}
// todo: BorshAccount?

// fuck yeah, now THIS WILL WORKS WITH WRAPPERS!!
impl<'a, T> AccountValidationExt for T
where
    T: Deref<Target = AccountInfo<'a>>,
{
    #[inline(always)]
    fn require<F: Fn(&Self) -> bool>(self, condition: F) -> Result<Self, ProgramError> {
        if !condition(&self) {
            return Err(ProgramError::InvalidArgument); // todo: what type of error should this be? sdk-custom?
        }
        Ok(self)
    }

    #[inline(always)]
    fn require_signer(self) -> Result<Self, ProgramError> {
        if !self.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(self)
    }

    #[inline(always)]
    fn require_mut(self) -> Result<Self, ProgramError> {
        if !self.is_writable {
            return Err(ProgramError::InvalidArgument)
        }
        Ok(self)
    }

    #[inline(always)]
    fn require_owner(self, program_id: &Pubkey) -> Result<Self, ProgramError> {
        if self.owner != program_id {
            return Err(ProgramError::IllegalOwner)
        }
        Ok(self)
    }

    #[inline(always)]
    fn require_adress(self, adress: &Pubkey) -> Result<Self, ProgramError> {
        if self.key != adress {
            return Err(ProgramError::IncorrectProgramId)
        }
        Ok(self)
    }

    #[inline(always)]
    fn require_system(self) -> Result<Self, ProgramError> {
        self.require_adress(&SYSTEM_PROGRAM_ID)
    }

    #[inline(always)]
    fn require_empty(self) -> Result<Self, ProgramError> {
        if !self.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(self)
    }

    #[inline(always)]
    fn require_bumped_pda(self, program_id: &Pubkey, seeds: &[&[u8]], bump: u8) -> Result<Self, ProgramError> {
        let bump = [bump]; // TODO DRY: move to utils
        let mut combined_seeds = Vec::with_capacity(seeds.len() + 1);
        combined_seeds.extend_from_slice(seeds);
        combined_seeds.push(&bump);

        let expected_pda = Pubkey::create_program_address(&combined_seeds, program_id)?;
        if self.key != &expected_pda {
            solana_program::msg!("PDA mismatch for account {}", self.key); // todo: custom shit for that
            return Err(ProgramError::InvalidArgument);
        }

        Ok(self)
    }
}

impl<'a, T> PdaExt for T
where
    T: Deref<Target = AccountInfo<'a>>,
{
    #[inline(always)]
    fn find_and_verify_pda(&self, seeds: &[&[u8]], program_id: &Pubkey) -> Result<u8, ProgramError> {
        let (expected_pda, bump) = Pubkey::find_program_address(seeds, program_id);
        if self.key != &expected_pda {
            solana_program::msg!("PDA mismatch for account {}", self.key); // todo: custom shit for that
            return Err(ProgramError::InvalidArgument);
        }

        Ok(bump)
    }
}

impl<'a> AccountBytemuckExt for AccountInfo<'a> {
    fn load<T: AccountState>(&self) -> Result<Ref<'_, T>, ProgramError> {
        let data = self.try_borrow_data()?;

        // Validate data
        T::try_from_bytes(&data)?;

        Ok(Ref::map(data, |data| {
            bytemuck::from_bytes(
                &data[DISC_SIZE .. T::SIZE]
            )
        }))
    }

    fn load_mut<T: AccountState>(&self) -> Result<RefMut<'_, T>, ProgramError> {
        let mut data = self.try_borrow_mut_data()?;

        // Validate data
        T::try_from_bytes_mut(&mut data)?;

        Ok(RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(
                &mut data[DISC_SIZE .. T::SIZE]
            )
        }))
    }

    fn with_mut<T: AccountState, F>(&self, f: F) -> Result<(), ProgramError>
    where F: FnOnce(&mut T)
    {
        let mut state = self.load_mut::<T>()?;
        f(&mut state);
        Ok(())
    }
}

pub trait InstructionArgs: AnyBitPattern {
    const SIZE: usize = std::mem::size_of::<Self>();

    fn parse_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        Ok(bytemuck::from_bytes(
            &data[..Self::SIZE]
        ))
    }
}
