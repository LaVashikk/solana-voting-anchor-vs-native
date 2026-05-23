use dummy_sdk::client::ClientInstruction;
use litesvm::{LiteSVM, types::TransactionResult};
use native_voter_cheap::{instructions::{close_candidate::{self, CloseCandidateArgs}, close_pull::{self, ClosePullArgs}, close_vote::{self, CloseVoteArgs}, create_candidate::{self, CreateCandidateArgs}, create_pull::{self, CreatePullArgs}, create_vote::{self, VotingArgs}}, state::voter::Voter};
use solana_sdk::{message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};


pub fn create_pull(svm: &mut LiteSVM, user: &Keypair, title: &str, description: &str, vote_price: u64) -> Pubkey {
    let pull = Keypair::new();
    let current_time = super::current_time();
    let args = CreatePullArgs::new(
        title,
        description,
        current_time,
        current_time + 10_000,
        vote_price,
    ).unwrap();

    let accounts = create_pull::client::CreatePullAccounts {
        payer: user.pubkey(),
        pull_key: pull.pubkey(),
    };

    let ix = args.build_ix(super::PROGRAM_ID, accounts);

    // Create a pull
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user, &pull], msg, svm.latest_blockhash());
    let x = svm.send_transaction(tx).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("CREATE PULL LOGS: {:#?}", x.logs);

    pull.pubkey()
}

pub fn create_candidate_raw(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: &Keypair, name: &str) -> TransactionResult {
    let args = CreateCandidateArgs::new(name).unwrap();
    let accounts = create_candidate::client::CreateCandidateAccounts {
        payer: user.pubkey(),
        pull_key: pull,
        candidate_key: candidate.pubkey(),
    };

    let ix = args.build_ix(super::PROGRAM_ID, accounts);

    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user, &candidate], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn create_candidate(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, name: &str) -> Pubkey {
    let candidate = Keypair::new();
    let x = create_candidate_raw(svm, user, pull, &candidate, name).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("CREATE CANDIDATES LOGS: {:#?}", x.logs);

    candidate.pubkey()
}

pub fn create_vote_raw(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> TransactionResult {
    let (voter_pda, _) = Pubkey::find_program_address(&Voter::get_seeds(&pull, &user.pubkey()), &super::PROGRAM_ID);
    let accounts = create_vote::client::VotingAccounts {
        voter: user.pubkey(),
        pull,
        candidate,
        voter_pda,
    };

    let ix = VotingArgs.build_ix(super::PROGRAM_ID, accounts);

    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn create_vote(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> Pubkey {
    let (voter_pda, _) = Pubkey::find_program_address(&Voter::get_seeds(&pull, &user.pubkey()), &super::PROGRAM_ID);
    let x = create_vote_raw(svm, user, pull, candidate).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("VOTED: {:#?}", x.logs);

    voter_pda
}

pub fn close_vote_raw(svm: &mut LiteSVM, bot: &Keypair, voter: Pubkey, voter_pda: Pubkey) -> TransactionResult {
    let accounts = close_vote::client::CloseVoteAccounts {
        voter,
        bot: bot.pubkey(),
        voter_tracker: voter_pda,
    };

    let ix = CloseVoteArgs.build_ix(super::PROGRAM_ID, accounts);
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

    let ix = CloseCandidateArgs.build_ix(super::PROGRAM_ID, accounts);
    let msg = Message::new(&[ix], Some(&creator.pubkey()));
    let tx = Transaction::new(&[creator], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}


pub fn close_pull_raw(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey) -> TransactionResult {
    let accounts = close_pull::client::ClosePullAccounts {
        creator: creator.pubkey(),
        pull,
    };

    let ix = ClosePullArgs.build_ix(super::PROGRAM_ID, accounts);
    let msg = Message::new(&[ix], Some(&creator.pubkey()));
    let tx = Transaction::new(&[creator], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
}

pub fn close_vote(svm: &mut LiteSVM, bot: &Keypair, voter: Pubkey, voter_pda: Pubkey) {
    let x = close_vote_raw(svm, bot, voter, voter_pda).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("CLOSE VOTE: {:#?}", x.logs);
}

pub fn close_candidate(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey, candidate: Pubkey) {
    let x = close_candidate_raw(svm, creator, pull, candidate).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("CLOSE CANDIDATE: {:#?}", x.logs);
}


pub fn close_pull(svm: &mut LiteSVM, creator: &Keypair, pull: Pubkey) {
    let x = close_pull_raw(svm, creator, pull).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("CLOSE PULL: {:#?}", x.logs);
}
