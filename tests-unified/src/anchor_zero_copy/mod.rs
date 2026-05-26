use solana_sdk::{message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use litesvm::{LiteSVM, types::TransactionResult};
use anchor_lang::{InstructionData, ToAccountMetas};
use crate::*;
use bytemuck::{Pod, Zeroable};

// --- Unified State Mappings ---

#[allow(dead_code)]
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Pull {
    pub creator: Pubkey,
    pub title: dummy_sdk::pod_types::FixedString<32>,
    pub description: dummy_sdk::pod_types::FixedString<32>,
    pub voting_start: i64,
    pub voting_end: i64,
    pub vote_price: u64,
    pub candidate_count: u64,
    pub last_candidate: dummy_sdk::pod_types::PodOption<Pubkey>,
    pub next_candidate_idx: u64,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Candidate {
    pub pull: Pubkey,
    pub name: dummy_sdk::pod_types::FixedString<32>,
    pub number_of_votes: u64,
    pub prev_candidate: dummy_sdk::pod_types::PodOption<Pubkey>,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Voter {
    pub voter_addr: Pubkey,
    pub voted_for: Pubkey,
    pub voting_end: i64,
}

impl Voter {
    pub const SIZE: usize = 80;
}

impl UnifiedState for Pull {
    fn try_from_bytes(data: &[u8]) -> Result<Self, anyhow::Error> {
        // Anchor zero_copy uses 8-byte discriminator
        let state: &zero_copy_anchor_vote::Pull = bytemuck::from_bytes(&data[8..]);
        Ok(Self {
            creator: state.creator,
            title: state.title,
            description: state.description,
            voting_start: state.voting_start,
            voting_end: state.voting_end,
            vote_price: state.vote_price,
            candidate_count: state.candidate_count,
            last_candidate: state.last_candidate,
            next_candidate_idx: state.next_candidate_idx,
        })
    }
}

impl UnifiedState for Candidate {
    fn try_from_bytes(data: &[u8]) -> Result<Self, anyhow::Error> {
        let state: &zero_copy_anchor_vote::Candidate = bytemuck::from_bytes(&data[8..]);
        Ok(Self {
            pull: state.pull,
            name: state.name,
            number_of_votes: state.number_of_votes,
            prev_candidate: state.prev_candidate,
        })
    }
}

impl UnifiedState for Voter {
    fn try_from_bytes(data: &[u8]) -> Result<Self, anyhow::Error> {
        let state: &zero_copy_anchor_vote::Voter = bytemuck::from_bytes(&data[8..]);
        Ok(Self {
            voter_addr: state.voter_addr,
            voted_for: state.voted_for,
            voting_end: state.voting_end,
        })
    }
}

// --- Caller Functions ---

pub fn create_pull_raw(svm: &mut LiteSVM, user: &Keypair, title: &str, description: &str, vote_price: u64) -> (Pubkey, TransactionResult) {
    let pull = Keypair::new();
    let start_time = current_time();
    let end_time = start_time + 10_000;

    let args = zero_copy_anchor_vote::instruction::CreatePull {
        title: title.to_string(),
        description: description.to_string(),
        voting_start: start_time,
        voting_end: end_time,
        vote_price,
    };
    let accounts = zero_copy_anchor_vote::accounts::CreatePull {
        pull: pull.pubkey(),
        payer: user.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };
    let ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: args.data(),
    };
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[user, &pull], msg, svm.latest_blockhash());
    (pull.pubkey(), svm.send_transaction(tx))
}

pub fn create_candidate_raw(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, name: &str) -> (Pubkey, TransactionResult) {
    let candidate = Keypair::new();
    let args = zero_copy_anchor_vote::instruction::CreateCandidate { name: name.to_string() };
    let accounts = zero_copy_anchor_vote::accounts::CreateCandidate {
        pull,
        candidate: candidate.pubkey(),
        payer: user.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };
    let ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: args.data(),
    };
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[user, &candidate], msg, svm.latest_blockhash());
    (candidate.pubkey(), svm.send_transaction(tx))
}

pub fn create_vote_raw(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> TransactionResult {
    let (voter_pda, _) = Pubkey::find_program_address(&[b"voter", pull.as_ref(), user.pubkey().as_ref()], &PROGRAM_ID);
    let args = zero_copy_anchor_vote::instruction::Voting {};
    let accounts = zero_copy_anchor_vote::accounts::Voting {
        pull,
        candidate,
        voter_tracker: voter_pda,
        payer: user.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };
    let ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: args.data(),
    };
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[user], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn ix_close_vote(bot: Pubkey, voter: Pubkey, voter_pda: Pubkey) -> solana_sdk::instruction::Instruction {
    let args = zero_copy_anchor_vote::instruction::CloseVote {};
    let accounts = zero_copy_anchor_vote::accounts::CloseVote {
        voter_tracker: voter_pda,
        voter,
        bot,
    };
    solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: args.data(),
    }
}

pub fn close_vote_raw(svm: &mut LiteSVM, bot: &Keypair, voter: Pubkey, voter_pda: Pubkey) -> TransactionResult {
    let ix = ix_close_vote(bot.pubkey(), voter, voter_pda);
    let msg = Message::new(&[ix], Some(&bot.pubkey()));
    let tx = Transaction::new(&[bot], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn close_candidate_raw(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey, candidate: Pubkey) -> TransactionResult {
    let args = zero_copy_anchor_vote::instruction::CloseCandidate {};
    let accounts = zero_copy_anchor_vote::accounts::CloseCandidate {
        pull,
        candidate,
        creator: creator.pubkey(),
    };
    let ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: args.data(),
    };
    let msg = Message::new(&[ix], Some(&creator.pubkey()));
    let tx = Transaction::new(&[creator], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn close_pull_raw(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey) -> TransactionResult {
    let args = zero_copy_anchor_vote::instruction::ClosePull {};
    let accounts = zero_copy_anchor_vote::accounts::ClosePull {
        pull,
        creator: creator.pubkey(),
    };
    let ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: args.data(),
    };
    let msg = Message::new(&[ix], Some(&creator.pubkey()));
    let tx = Transaction::new(&[creator], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn create_pull(svm: &mut LiteSVM, user: &Keypair, title: &str, description: &str, vote_price: u64) -> Pubkey {
    let (pubkey, res) = create_pull_raw(svm, user, title, description, vote_price);
    let x = res.map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("CREATE PULL LOGS: {:#?}", x.logs);
    pubkey
}

pub fn create_candidate(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, name: &str) -> Pubkey {
    let (pubkey, res) = create_candidate_raw(svm, user, pull, name);
    let x = res.map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("CREATE CANDIDATES LOGS: {:#?}", x.logs);
    pubkey
}

pub fn create_vote(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> Pubkey {
    let (voter_pda, _) = Pubkey::find_program_address(&[b"voter", pull.as_ref(), user.pubkey().as_ref()], &PROGRAM_ID);
    let x = create_vote_raw(svm, user, pull, candidate).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("VOTED: {:#?}", x.logs);
    voter_pda
}

pub fn close_vote(svm: &mut LiteSVM, bot: &Keypair, voter: Pubkey, voter_pda: Pubkey) {
    close_vote_raw(svm, bot, voter, voter_pda).unwrap();
}

pub fn close_candidate(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey, candidate: Pubkey) {
    close_candidate_raw(svm, creator, pull, candidate).unwrap();
}

pub fn close_pull(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey) {
    close_pull_raw(svm, creator, pull).unwrap();
}

pub fn native_get_all_candidate(svm: &LiteSVM, pull_pubkey: &Pubkey) -> Vec<Candidate> {
    let pull = read_data::<Pull>(svm, pull_pubkey);
    let mut result = Vec::new();
    let mut prev_candidate: Option<Pubkey> = pull.last_candidate.into();
    while let Some(candidate_pubkey) = prev_candidate {
        let candidate = read_data::<Candidate>(svm, &candidate_pubkey);
        prev_candidate = candidate.prev_candidate.into();
        result.push(candidate);
    }
    result
}
