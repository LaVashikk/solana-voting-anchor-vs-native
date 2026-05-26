use solana_sdk::{message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use litesvm::{LiteSVM, types::TransactionResult};
use anchor_lang::{InstructionData, ToAccountMetas, AccountDeserialize};
use crate::common::*;

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
    pub next_candidate_idx: u64,
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
        let anchor = anchor_vote::Pull::try_deserialize(&mut &data[..])?;
        Ok(Self {
            creator: anchor.creator,
            title: UnifiedString(anchor.metadata.title),
            description: UnifiedString(anchor.metadata.description),
            voting_start: anchor.metadata.voting_start,
            voting_end: anchor.metadata.voting_end,
            vote_price: anchor.vote_price,
            candidate_count: anchor.candidate_count,
            last_candidate: PodOption::none(),
            next_candidate_idx: anchor.next_candidate_idx,
        })
    }
}

impl UnifiedState for Candidate {
    fn try_from_bytes(data: &[u8]) -> Result<Self, anyhow::Error> {
        let anchor = anchor_vote::Candidate::try_deserialize(&mut &data[..])?;
        Ok(Self {
            pull: anchor.pull_key,
            name: UnifiedString(anchor.name),
            number_of_votes: anchor.number_of_votes,
            prev_candidate: PodOption::none(),
        })
    }
}

impl UnifiedState for Voter {
    fn try_from_bytes(data: &[u8]) -> Result<Self, anyhow::Error> {
        let anchor = anchor_vote::Voter::try_deserialize(&mut &data[..])?;
        Ok(Self {
            voter_addr: anchor.voter_addr,
            voted_for: anchor.voted_for,
            voting_end: anchor.voting_end,
        })
    }
}

// --- Caller Functions ---

const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");

pub fn create_pull_raw(svm: &mut LiteSVM, user: &Keypair, title: &str, description: &str, vote_price: u64) -> (Pubkey, TransactionResult) {
    let (pull_pda, _) = Pubkey::find_program_address(&[b"pull", title.as_bytes()], &PROGRAM_ID);
    let start_time = current_time();
    let end_time = start_time + 10_000;

    let args = anchor_vote::instruction::CreatePull {
        metadata: anchor_vote::PullMetadata {
            title: title.to_string(),
            description: description.to_string(),
            voting_start: start_time,
            voting_end: end_time,
        },
        vote_price,
    };
    let accounts = anchor_vote::accounts::CreatePull {
        pull_pda,
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
    (pull_pda, svm.send_transaction(tx))
}

pub fn create_candidate_raw(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, name: &str) -> (Pubkey, TransactionResult) {
    let pull_account = svm.get_account(&pull).expect("Pull account not found");
    let pull_data = anchor_vote::Pull::try_deserialize(&mut &pull_account.data[..]).unwrap();
    let (candidate_pda, _) = Pubkey::find_program_address(
        &[b"candidate", pull.as_ref(), &pull_data.next_candidate_idx.to_le_bytes()],
        &PROGRAM_ID
    );
    let args = anchor_vote::instruction::CreateCandidate { name: name.to_string() };
    let accounts = anchor_vote::accounts::CreateCandidate {
        pull,
        candidate_pda,
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
    (candidate_pda, svm.send_transaction(tx))
}

pub fn create_vote_raw(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> TransactionResult {
    let (voter_pda, _) = Pubkey::find_program_address(&[b"voter", pull.as_ref(), user.pubkey().as_ref()], &PROGRAM_ID);
    let args = anchor_vote::instruction::Voting {};
    let accounts = anchor_vote::accounts::Voting {
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
    let args = anchor_vote::instruction::CloseVoting {};
    let accounts = anchor_vote::accounts::CloseVoting {
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
    let args = anchor_vote::instruction::CloseCandidate {};
    let accounts = anchor_vote::accounts::CloseCandidate {
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
    let args = anchor_vote::instruction::ClosePull {};
    let accounts = anchor_vote::accounts::ClosePull {
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
    res.unwrap();
    pubkey
}

pub fn create_candidate(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, name: &str) -> Pubkey {
    let (pubkey, res) = create_candidate_raw(svm, user, pull, name);
    res.unwrap();
    pubkey
}

pub fn create_vote(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> Pubkey {
    let (voter_pda, _) = Pubkey::find_program_address(&[b"voter", pull.as_ref(), user.pubkey().as_ref()], &PROGRAM_ID);
    create_vote_raw(svm, user, pull, candidate).unwrap();
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
    for i in 0..pull.next_candidate_idx {
        let (candidate_pda, _) = Pubkey::find_program_address(
            &[b"candidate", pull_pubkey.as_ref(), &i.to_le_bytes()],
            &PROGRAM_ID
        );
        if svm.get_account(&candidate_pda).is_some() {
            result.push(read_data::<Candidate>(svm, &candidate_pda));
        }
    }
    result
}
