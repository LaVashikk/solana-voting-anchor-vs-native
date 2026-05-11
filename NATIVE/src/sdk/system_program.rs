use solana_program::{
    program::invoke,
    rent::Rent,
    sysvar::Sysvar,
};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use solana_program::program::invoke_signed;
use solana_system_interface::instruction;

use crate::sdk::{AccountInfoExt, Discriminator};

// TODO: a relly VERY WIP
pub trait SystemCpiExt<'a> {
    fn create_raw_account_cpi(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        space: usize,
        owner: &Pubkey,
    ) -> Result<(), ProgramError>;

    fn create_sdk_account_cpi<T: Discriminator>(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        owner: &Pubkey,
    ) -> Result<(), ProgramError>;

    fn create_raw_pda_cpi(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        space: usize,
        seeds: &[&[u8]],
        bump: u8,
        program_id: &Pubkey,
    ) -> Result<(), ProgramError>;

    fn create_sdk_pda_cpi<T: Discriminator>(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        seeds: &[&[u8]],
        bump: u8,
        program_id: &Pubkey,
    ) -> Result<(), ProgramError>;

    // Sugar for 'in-buissnes-logic' uses, or whatever
    fn checked_create_sdk_pda_cpi<T: Discriminator>(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError>;
}

impl<'a> SystemCpiExt<'a> for AccountInfo<'a> {
    fn create_raw_account_cpi(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        space: usize,
        owner: &Pubkey,
    ) -> Result<(), ProgramError> {
        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(space);

        invoke(
            &instruction::create_account(
                payer.key,
                self.key,
                required_lamports,
                space as u64,
                owner,
            ),
            &[payer.clone(), self.clone(), system_program.clone()],
        )
    }

    #[inline(always)]
    fn create_sdk_account_cpi<T: Discriminator>(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        owner: &Pubkey,
    ) -> Result<(), ProgramError> {
        self.create_raw_account_cpi(payer, system_program, T::SIZE, owner)?;
        let mut data = self.try_borrow_mut_data()?;

        // Should be empty
        if data[0] != 0 {
            return Err(ProgramError::InvalidAccountData)
        }

        // And now write the discriminator
        data[0] = T::DISCRIMINATOR;
        // 1-7 empty space for padding. We can store anything at all here
        data[1..8].copy_from_slice(b"NativeS");

        Ok(())
    }

    #[inline(always)]
    fn create_raw_pda_cpi(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        space: usize,
        seeds: &[&[u8]],
        bump: u8,
        program_id: &Pubkey,
    ) -> Result<(), ProgramError> {
        let bump = [bump];
        let mut combined_seeds = Vec::with_capacity(seeds.len() + 1);
        combined_seeds.extend_from_slice(seeds);
        combined_seeds.push(&bump);

        let required_lamports = Rent::get()?.minimum_balance(space);

        let ix = instruction::create_account(
            payer.key,
            self.key,
            required_lamports,
            space as u64,
            program_id
        );

        let accounts = [payer.clone(), self.clone(), system_program.clone()];

        invoke_signed(&ix, &accounts, &[&combined_seeds])?;

        Ok(())
    }

    #[inline(always)]
    fn create_sdk_pda_cpi<T: Discriminator>(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        seeds: &[&[u8]],
        bump: u8,
        program_id: &Pubkey,
    ) -> Result<(), ProgramError> {
        self.create_raw_pda_cpi(payer, system_program, T::SIZE, seeds, bump, program_id)?;
        let mut data = self.try_borrow_mut_data()?;

        // Should be empty
        if data[0] != 0 {
            return Err(ProgramError::InvalidAccountData)
        }

        // And now write the discriminator
        data[0] = T::DISCRIMINATOR;
        Ok(())
    }

    #[inline(always)]
    fn checked_create_sdk_pda_cpi<T: Discriminator>(
        &self,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let bump = self.assert_pda(seeds, program_id)?;
        self.create_sdk_pda_cpi::<T>(payer, system_program, seeds, bump, program_id)?;
        Ok(bump)
    }
}
