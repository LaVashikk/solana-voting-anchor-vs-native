use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    program_error::ProgramError
};
use bytemuck::Pod;

pub trait IntoAccountMeta {
    fn mut_signer(self) -> AccountMeta;
    fn mut_owned(self) -> AccountMeta;
    fn readonly_signer(self) -> AccountMeta;
    fn readonly(self) -> AccountMeta;
}

impl IntoAccountMeta for Pubkey {
    fn mut_signer(self) -> AccountMeta { AccountMeta::new(self, true) }
    fn mut_owned(self) -> AccountMeta { AccountMeta::new(self, false) }
    fn readonly_signer(self) -> AccountMeta { AccountMeta::new_readonly(self, true) }
    fn readonly(self) -> AccountMeta { AccountMeta::new_readonly(self, false) }
}

pub trait ClientInstruction: Pod {
    type Accounts;
    const IX_TAG: u64;

    fn accounts_to_metas(accounts: &Self::Accounts) -> Vec<AccountMeta>;

    // ----- inner methods -----
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = vec![0u8; 8 + std::mem::size_of::<Self>()];
        data[0..8].copy_from_slice(&Self::IX_TAG.to_le_bytes());
        data[8..].copy_from_slice(bytemuck::bytes_of(self));
        data
    }

    fn from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < std::mem::size_of::<Self>() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        Ok(bytemuck::from_bytes(
            &data[..std::mem::size_of::<Self>()]
        ))
    }

    fn build_ix(&self, program_id: Pubkey, accounts: Self::Accounts) -> Instruction {
        let data = self.to_bytes();
        Instruction {
            program_id,
            accounts: Self::accounts_to_metas(&accounts),
            data,
        }
    }
}
