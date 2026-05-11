use litesvm::LiteSVM;
use solana_sdk::{message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

use crate::common::{ix_create_candidate, ix_create_pull, ix_voting};


pub fn create_pull(svm: &mut LiteSVM, user: &Keypair, title: &str, description: &str, vote_price: u64) -> (Pubkey, u8) {
    let (ix, pull_pda, bump) = ix_create_pull(user.pubkey(), title.to_string(), description.to_string(), vote_price);

    // Create a pull
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg, svm.latest_blockhash());
    let x = svm.send_transaction(tx).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();
    println!("LOGS: {:#?}", x.logs);

    (pull_pda, bump)
}

pub fn create_candidate(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, name: &str, idx: u64) -> (Pubkey, u8) {
    let (ix, candidate_pda, bump) = ix_create_candidate(user.pubkey(), pull, name, idx);

    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg, svm.latest_blockhash());
    svm.send_transaction(tx).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();

    (candidate_pda, bump)
}

pub fn voting(svm: &mut LiteSVM, user: &Keypair, pull: Pubkey, candidate: Pubkey) -> Pubkey {
    let (ix, voter_pda) = ix_voting(user.pubkey(), pull, candidate);

    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg, svm.latest_blockhash());
    svm.send_transaction(tx).map_err(|e| {eprintln!("LOGS: {:#?}", e.meta.logs); e}).unwrap();

    voter_pda
}
