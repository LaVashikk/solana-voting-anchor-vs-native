mod common;
use anchor_lang::AccountDeserialize;
use common::*;
use solana_sdk::{message::Message, signer::Signer, transaction::Transaction};

#[test]
#[should_panic]
fn cross_poll_voting_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");

    let (pull_1, _) = create_pull(&mut svm, &user, "Poll 1", "Desc 1", 0);
    let (pull_2, _) = create_pull(&mut svm, &user, "Poll 2", "Desc 2", 0);

    // Candidate belongs to poll_2
    let (candidate_2, _) = create_candidate(&mut svm, &user, pull_2, "Python", 0);

    // Try to vote in poll_1 with candidate from poll_2
    let (ix, _) = ix_voting(user.pubkey(), pull_1, candidate_2);

    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg, svm.latest_blockhash());
    svm.send_transaction(tx).unwrap(); // Should fail: candidate.pull != pull_1
}

#[test]
#[should_panic]
fn candidate_index_mismatch_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");
    let (pull, _) = create_pull(&mut svm, &user, "Best language", "Test", 0);

    // pull.candidate_count is 0, but we pass idx 5
    // Should fail if the contract properly checks: idx == pull.candidate_count
    let _ = create_candidate(&mut svm, &user, pull, "Rust", 5);
}

#[test]
#[should_panic]
fn duplicate_candidate_pda_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");
    let (pull, _) = create_pull(&mut svm, &user, "Best language", "Test", 0);

    // Create first candidate at idx 0
    let _ = create_candidate(&mut svm, &user, pull.clone(), "Rust", 0);

    // Try to create another candidate at the SAME idx 0
    // Should fail: account already initialized (PDA collision)
    let _ = create_candidate(&mut svm, &user, pull, "Python", 0);
}

#[test]
fn voting_after_end_test() {
    let (mut svm, creator) = init_svm_env("anchor_vote");
    let user1 = create_user(&mut svm);
    let user2 = create_user(&mut svm);

    let (pull_pda, _) = create_pull(&mut svm, &creator, "Best programming language", "This is a test pull", 0);
    let (candidate_1, _) = create_candidate(&mut svm, &creator, pull_pda.clone(), "Rust", 0);

    let check_candidate_votes = |svm: &litesvm::LiteSVM| {
        let candidate_account = svm.get_account(&candidate_1).unwrap();
        let candidate_data = anchor_vote::Candidate::try_deserialize(&mut candidate_account.data.as_slice()).unwrap(); // todo: improve it, move to common
        candidate_data.number_of_votes
    };

    // Voting started
    set_svm_time(&mut svm, current_time() + 100);
    voting(&mut svm, &user1, pull_pda.clone(), candidate_1.clone());
    assert_eq!(check_candidate_votes(&svm), 1);

    set_svm_time(&mut svm, current_time() + 100_000);

    // not allowed!
    let (ix, _) = ix_voting(user2.pubkey(), pull_pda, candidate_1);

    let msg = Message::new(&[ix], Some(&user2.pubkey()));
    let tx = Transaction::new(&[&user2], msg, svm.latest_blockhash());
    let res = svm.send_transaction(tx);

    assert_eq!(check_candidate_votes(&svm), 1);
    assert!(res.is_err());

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("VotingAlreadyEnded")),
        "Contract accepted voting after closing! Logs: {:#?}",
        err.meta.logs
    );
}
