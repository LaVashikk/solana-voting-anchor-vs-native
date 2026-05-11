use litesvm::LiteSVM;
use native_voter_cheap::{instructions::{create_candidate::{self, CreateCandidateArgs, client::CreateCandidateAccounts}, create_pull::{self, CreatePullArgs}, voting::{self, VotingArgs}}, sdk::{AccountState, off_chain::ClientInstruction}, state::{pull::Pull, voter::Voter}};
use solana_program::rent::Rent;
use solana_sdk::{message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};


pub fn create_pull(svm: &mut LiteSVM, user: &Keypair, title: &str, description: &str, vote_price: u64) -> Pubkey {
    let args = CreatePullArgs::new(title, description, 0, 0, vote_price).unwrap(); // todo: time
    let pull = Keypair::new();
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

pub fn create_candidate(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, name: &str) -> Pubkey {
    let args = CreateCandidateArgs::new(name).unwrap();
    let candidate = Keypair::new();
    let accounts = create_candidate::client::CreateCandidateAccounts {
        payer: user.pubkey(),
        pull_key: pull,
        candidate_key: candidate.pubkey(),
    };

    let ix = args.build_ix(super::PROGRAM_ID, accounts);

    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user, &candidate], msg, svm.latest_blockhash());
    let x = svm.send_transaction(tx).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("CREATE CANDIDATES LOGS: {:#?}", x.logs);

    candidate.pubkey()
}

pub fn voting(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> Pubkey {
    let (voter_pda, _) = Pubkey::find_program_address(&Voter::get_seeds(&pull, &user.pubkey()), &super::PROGRAM_ID);
    let accounts = voting::client::VotingAccounts {
        voter: user.pubkey(),
        pull,
        candidate,
        voter_pda,
    };

    let ix = VotingArgs.build_ix(super::PROGRAM_ID, accounts);

    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg, svm.latest_blockhash());
    let x = svm.send_transaction(tx).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("VOTED: {:#?}", x.logs);

    voter_pda
}
