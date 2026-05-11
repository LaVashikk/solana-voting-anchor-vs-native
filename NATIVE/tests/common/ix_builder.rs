use anchor_lang::{InstructionData, ToAccountMetas, prelude::system_program};
use solana_sdk::{message::Instruction, pubkey::Pubkey};

// UNUSED, OUTDATED, REMOVE LATER

pub fn ix_create_pull(
    payer: Pubkey,
    title: String,
    description: String,
    vote_price: u64,
) -> (Instruction, Pubkey, u8) {

    let (pull_pda, bump) = native_voter_cheap::Pull::find_pda(&title);
    let current_time = super::current_time();

    let metadata = native_voter_cheap::PullMetadata {
        title,
        description,
        voting_start: current_time,
        voting_end: current_time + 10_000,
    };

    let accounts = native_voter_cheap::accounts::CreatePull {
        pull_pda,
        payer,
        system_program: system_program::ID,
    };

    (
        Instruction {
            program_id: native_voter_cheap::ID,
            accounts: accounts.to_account_metas(None),
            data: native_voter_cheap::instruction::CreatePull { metadata, vote_price }.data(),
        },
        pull_pda,
        bump,
    )
}

pub fn ix_create_candidate(
    payer: Pubkey,
    pull_pda: Pubkey,
    name: &str,
    idx: u64, // in real client, we should read candidate_count from pull data
    ) -> (Instruction, Pubkey, u8)
{
    let (candidate_pda, bump) = native_voter_cheap::Candidate::find_pda(&pull_pda, idx.to_le_bytes().as_ref());

    let accounts = native_voter_cheap::accounts::CreateCandidate {
        payer,
        pull: pull_pda,
        candidate_pda,
        system_program: system_program::ID,
    };

    let ix_data = native_voter_cheap::instruction::CreateCandidate {
        name: name.to_string(),
    }.data();

    let ix = Instruction {
        program_id: native_voter_cheap::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data,
    };

    (ix, candidate_pda, bump)
}

pub fn ix_voting(
    payer: Pubkey,
    pull: Pubkey,
    candidate: Pubkey,
) -> (Instruction, Pubkey) {
    let (voter_pda, _) = native_voter_cheap::Voter::find_pda(&pull, &payer);
    let accounts = native_voter_cheap::accounts::Voting {
        pull,
        payer,
        candidate,
        voter_tracker: voter_pda,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: native_voter_cheap::ID,
        accounts: accounts.to_account_metas(None),
        data: native_voter_cheap::instruction::Voting.data(),
    };

    (ix, voter_pda)
}

pub fn ix_close_pull(
    creator: Pubkey,
    pull_pda: Pubkey,
) -> Instruction {
    let accounts = native_voter_cheap::accounts::ClosePull {
        creator,
        pull: pull_pda,
    };

    let ix = Instruction {
        program_id: native_voter_cheap::ID,
        accounts: accounts.to_account_metas(None),
        data: native_voter_cheap::instruction::ClosePull.data(),
    };

    ix
}

pub fn ix_close_candidate(
    creator: Pubkey,
    pull_pda: Pubkey,
    candidate_pda: Pubkey,
) -> Instruction {
    let accounts = native_voter_cheap::accounts::CloseCandidate {
        creator,
        pull: pull_pda,
        candidate: candidate_pda,
    };

    let ix = Instruction {
        program_id: native_voter_cheap::ID,
        accounts: accounts.to_account_metas(None),
        data: native_voter_cheap::instruction::CloseCandidate.data(),
    };

    ix
}


pub fn ix_close_voting(
    payer: Pubkey,
    voter: Pubkey,
    voter_tracker: Pubkey,
) -> Instruction {
    let accounts = native_voter_cheap::accounts::CloseVoting {
        bot: payer,
        voter,
        voter_tracker,
    };

    let ix = Instruction {
        program_id: native_voter_cheap::ID,
        accounts: accounts.to_account_metas(None),
        data: native_voter_cheap::instruction::CloseVoting.data(),
    };

    ix
}
