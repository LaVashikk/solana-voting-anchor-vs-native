use anchor_lang::{InstructionData, ToAccountMetas, prelude::system_program};
use solana_sdk::{message::Instruction, pubkey::Pubkey};

pub fn ix_create_pull(
    payer: Pubkey,
    title: String,
    description: String,
    vote_price: u64,
) -> (Instruction, Pubkey, u8) {

    let (pull_pda, bump) = anchor_vote::Pull::find_pda(&title);
    let current_time = super::current_time();

    let metadata = anchor_vote::PullMetadata {
        title,
        description,
        voting_start: current_time,
        voting_end: current_time + 10_000,
    };

    let accounts = anchor_vote::accounts::CreatePull {
        pull_pda,
        payer,
        system_program: system_program::ID,
    };

    (
        Instruction {
            program_id: anchor_vote::ID,
            accounts: accounts.to_account_metas(None),
            data: anchor_vote::instruction::CreatePull { metadata, vote_price }.data(),
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
    let (candidate_pda, bump) = anchor_vote::Candidate::find_pda(&pull_pda, idx.to_le_bytes().as_ref());

    let accounts = anchor_vote::accounts::CreateCandidate {
        payer,
        pull: pull_pda,
        candidate_pda,
        system_program: system_program::ID,
    };

    let ix_data = anchor_vote::instruction::CreateCandidate {
        name: name.to_string(),
    }.data();

    let ix = Instruction {
        program_id: anchor_vote::ID,
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
    let (voter_pda, _) = anchor_vote::Voter::find_pda(&pull, &payer);
    let accounts = anchor_vote::accounts::Voting {
        pull,
        payer,
        candidate,
        voter_tracker: voter_pda,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: anchor_vote::ID,
        accounts: accounts.to_account_metas(None),
        data: anchor_vote::instruction::Voting.data(),
    };

    (ix, voter_pda)
}

pub fn ix_close_pull(
    creator: Pubkey,
    pull_pda: Pubkey,
) -> Instruction {
    let accounts = anchor_vote::accounts::ClosePull {
        creator,
        pull: pull_pda,
    };

    let ix = Instruction {
        program_id: anchor_vote::ID,
        accounts: accounts.to_account_metas(None),
        data: anchor_vote::instruction::ClosePull.data(),
    };

    ix
}

pub fn ix_close_candidate(
    creator: Pubkey,
    pull_pda: Pubkey,
    candidate_pda: Pubkey,
) -> Instruction {
    let accounts = anchor_vote::accounts::CloseCandidate {
        creator,
        pull: pull_pda,
        candidate: candidate_pda,
    };

    let ix = Instruction {
        program_id: anchor_vote::ID,
        accounts: accounts.to_account_metas(None),
        data: anchor_vote::instruction::CloseCandidate.data(),
    };

    ix
}


pub fn ix_close_voting(
    payer: Pubkey,
    voter: Pubkey,
    voter_tracker: Pubkey,
) -> Instruction {
    let accounts = anchor_vote::accounts::CloseVoting {
        bot: payer,
        voter,
        voter_tracker,
    };

    let ix = Instruction {
        program_id: anchor_vote::ID,
        accounts: accounts.to_account_metas(None),
        data: anchor_vote::instruction::CloseVoting.data(),
    };

    ix
}
