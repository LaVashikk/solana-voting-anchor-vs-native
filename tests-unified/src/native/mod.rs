use solana_sdk::{message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use litesvm::{LiteSVM, types::TransactionResult};
use dummy_sdk::client::ClientInstruction;
use native_voter_cheap::state::{pull::Pull as NativePull, candidate::Candidate as NativeCandidate, voter::Voter as NativeVoter};
use native_voter_cheap::instructions::{
    create_pull::{self, CreatePullArgs},
    create_candidate::{self, CreateCandidateArgs},
    create_vote::{self, VotingArgs},
    close_pull::{self, ClosePullArgs},
    close_candidate::{self, CloseCandidateArgs},
    close_vote::{self, CloseVoteArgs},
};
use crate::*;

// --- Unified State Mappings ---

#[allow(dead_code)]
pub struct Pull {
    pub creator: Pubkey,
    pub title: UnifiedString,
    pub description: UnifiedString,
    pub voting_start: i64,
    pub voting_end: i64,
    pub vote_price: u64,
    pub candidate_count: u64,
    pub last_candidate: PodOption<Pubkey>,
}

#[allow(dead_code)]
pub struct Candidate {
    pub pull: Pubkey,
    pub name: UnifiedString,
    pub number_of_votes: u64,
    pub prev_candidate: PodOption<Pubkey>,
}

#[allow(dead_code)]
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
        use dummy_sdk::accounts::pod::PodAccountData;
        let native = <NativePull as PodAccountData>::try_from_bytes(data).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(Self {
            creator: native.creator,
            title: UnifiedString(native.title.as_str_lossy().to_string()),
            description: UnifiedString(native.description.as_str_lossy().to_string()),
            voting_start: native.voting_start,
            voting_end: native.voting_end,
            vote_price: native.vote_price,
            candidate_count: native.candidate_count,
            last_candidate: PodOption(native.last_candidate.into()),
        })
    }
}

impl UnifiedState for Candidate {
    fn try_from_bytes(data: &[u8]) -> Result<Self, anyhow::Error> {
        use dummy_sdk::accounts::pod::PodAccountData;
        let native = <NativeCandidate as PodAccountData>::try_from_bytes(data).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(Self {
            pull: native.pull,
            name: UnifiedString(native.name.as_str_lossy().to_string()),
            number_of_votes: native.number_of_votes,
            prev_candidate: PodOption(native.prev_candidate.into()),
        })
    }
}

impl UnifiedState for Voter {
    fn try_from_bytes(data: &[u8]) -> Result<Self, anyhow::Error> {
        use dummy_sdk::accounts::pod::PodAccountData;
        let native = <NativeVoter as PodAccountData>::try_from_bytes(data).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(Self {
            voter_addr: native.voter_addr,
            voted_for: native.voted_for,
            voting_end: native.voting_end,
        })
    }
}

// --- Caller Functions ---

pub fn create_pull_raw(svm: &mut LiteSVM, user: &Keypair, title: &str, description: &str, vote_price: u64) -> (Pubkey, TransactionResult) {
    let pull = Keypair::new();
    let start_time = current_time();
    let end_time = start_time + 10_000;

    let args = CreatePullArgs::new(title, description, start_time, end_time, vote_price).unwrap();
    let accounts = create_pull::client::CreatePullAccounts {
        payer: user.pubkey(),
        pull_key: pull.pubkey(),
    };

    let ix = args.build_ix(PROGRAM_ID, accounts);
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[user, &pull], msg, svm.latest_blockhash());
    (pull.pubkey(), svm.send_transaction(tx))
}

pub fn create_candidate_raw(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, name: &str) -> (Pubkey, TransactionResult) {
    let candidate = Keypair::new();
    let args = CreateCandidateArgs::new(name).unwrap();
    let accounts = create_candidate::client::CreateCandidateAccounts {
        payer: user.pubkey(),
        pull_key: pull,
        candidate_key: candidate.pubkey(),
    };

    let ix = args.build_ix(PROGRAM_ID, accounts);
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[user, &candidate], msg, svm.latest_blockhash());
    (candidate.pubkey(), svm.send_transaction(tx))
}

pub fn create_vote_raw(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> TransactionResult {
    let (voter_pda, _) = Pubkey::find_program_address(&NativeVoter::get_seeds(&pull, &user.pubkey()), &PROGRAM_ID);
    let accounts = create_vote::client::VotingAccounts {
        voter: user.pubkey(),
        pull,
        candidate,
        voter_pda,
    };

    let ix = VotingArgs.build_ix(PROGRAM_ID, accounts);
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[user], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn ix_close_vote(bot: Pubkey, voter: Pubkey, voter_pda: Pubkey) -> solana_sdk::instruction::Instruction {
    let accounts = close_vote::client::CloseVoteAccounts {
        voter,
        bot,
        voter_tracker: voter_pda,
    };
    CloseVoteArgs.build_ix(PROGRAM_ID, accounts)
}

pub fn close_vote_raw(svm: &mut LiteSVM, bot: &Keypair, voter: Pubkey, voter_pda: Pubkey) -> TransactionResult {
    let ix = ix_close_vote(bot.pubkey(), voter, voter_pda);
    let msg = Message::new(&[ix], Some(&bot.pubkey()));
    let tx = Transaction::new(&[bot], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn close_candidate_raw(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey, candidate: Pubkey) -> TransactionResult {
    let accounts = close_candidate::client::CloseCandidateAccounts {
        creator: creator.pubkey(),
        pull,
        candidate,
    };

    let ix = CloseCandidateArgs.build_ix(PROGRAM_ID, accounts);
    let msg = Message::new(&[ix], Some(&creator.pubkey()));
    let tx = Transaction::new(&[creator], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn close_pull_raw(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey) -> TransactionResult {
    let accounts = close_pull::client::ClosePullAccounts {
        creator: creator.pubkey(),
        pull,
    };

    let ix = ClosePullArgs.build_ix(PROGRAM_ID, accounts);
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
    let (voter_pda, _) = Pubkey::find_program_address(&NativeVoter::get_seeds(&pull, &user.pubkey()), &PROGRAM_ID);
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
    let mut prev_candidate = pull.last_candidate.0;
    while let Some(candidate_pubkey) = prev_candidate {
        let candidate = read_data::<Candidate>(svm, &candidate_pubkey);
        prev_candidate = candidate.prev_candidate.0;
        result.push(candidate);
    }
    result
}
