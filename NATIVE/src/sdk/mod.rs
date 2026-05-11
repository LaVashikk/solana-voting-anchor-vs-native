use std::{cell::{Ref, RefMut}, mem};

use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use bytemuck::{AnyBitPattern, Pod};
use solana_program::msg;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;

#[cfg(not(target_os = "solana"))]
pub mod off_chain;

pub mod system_program;
pub mod utils;

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
pub trait AccountInfoExt<'a> {
    fn assert_signer(&self) -> Result<&Self, ProgramError>;
    fn assert_mut(&self) -> Result<&Self, ProgramError>;
    fn assert_owner(&self, program_id: &Pubkey) -> Result<&Self, ProgramError>;
    fn assert_adress(&self, adress: &Pubkey) -> Result<&Self, ProgramError>;
    fn assert_system(&self) -> Result<&Self, ProgramError>;
    fn assert_empty(&self) -> Result<&Self, ProgramError>;
    fn assert_bumped_pda(&self, program_id: &Pubkey, seeds: &[&[u8]], bump: u8) -> Result<&Self, ProgramError>;
    // todo: ngl, pretty shitty name. should change it
    fn assert_pda(&self, seeds: &[&[u8]], program_id: &Pubkey) -> Result<u8, ProgramError>;

    fn load<T: AccountState>(&'a self) -> Result<Ref<'a, T>, ProgramError>;
    fn load_mut<T: AccountState>(&'a self) -> Result<RefMut<'a, T>, ProgramError>;
}

impl<'a> AccountInfoExt<'a> for AccountInfo<'a> {
    fn assert_signer(&self) -> Result<&Self, ProgramError> {
        if !self.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(self)
    }

    fn assert_mut(&self) -> Result<&Self, ProgramError> {
        if !self.is_writable {
            return Err(ProgramError::InvalidArgument)
        }
        Ok(self)
    }

    fn assert_owner(&self, program_id: &Pubkey) -> Result<&Self, ProgramError> {
        if self.owner != program_id {
            return Err(ProgramError::IllegalOwner)
        }
        Ok(self)
    }

    fn assert_adress(&self, adress: &Pubkey) -> Result<&Self, ProgramError> {
        if self.key != adress {
            return Err(ProgramError::IncorrectProgramId)
        }
        Ok(self)
    }

    fn assert_system(&self) -> Result<&Self, ProgramError> {
        self.assert_adress(&SYSTEM_PROGRAM_ID)
    }

    fn assert_empty(&self) -> Result<&Self, ProgramError> {
        if !self.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(self)
    }

    fn assert_pda(&self, seeds: &[&[u8]], program_id: &Pubkey) -> Result<u8, ProgramError> {
        let (expected_pda, bump) = Pubkey::find_program_address(seeds, program_id);
        if self.key != &expected_pda {
            solana_program::msg!("PDA mismatch for account {}", self.key); // todo: custom shit for that
            return Err(ProgramError::InvalidArgument);
        }

        Ok(bump)
    }

    fn assert_bumped_pda(&self, program_id: &Pubkey, seeds: &[&[u8]], bump: u8) -> Result<&Self, ProgramError> {
        let bump = [bump]; // DRY: move to utils
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


    fn load<T: AccountState>(&'a self) -> Result<Ref<'a, T>, ProgramError> {
        let data = self.try_borrow_data()?;

        // Validate data
        T::try_from_bytes(&data)?;

        Ok(Ref::map(data, |data| {
            bytemuck::from_bytes(
                &data[DISC_SIZE .. T::SIZE]
            )
        }))
    }

    fn load_mut<T: AccountState>(&'a self) -> Result<RefMut<'a, T>, ProgramError> {
        let mut data = self.try_borrow_mut_data()?;

        // Validate data
        T::try_from_bytes_mut(&mut data)?;

        Ok(RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(
                &mut data[DISC_SIZE .. T::SIZE]
            )
        }))
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
