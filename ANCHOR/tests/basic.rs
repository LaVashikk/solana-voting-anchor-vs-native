use anchor_lang::{AccountDeserialize, Space};
use solana_sdk::{message::Message, native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer, transaction::Transaction};

mod common;
use common::*;

#[test]
fn create_pull_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");
    let (ix, pull_pda, bump) = ix_create_pull(user.pubkey(), "Test Pull".to_string(), "This is a test pull".to_string(), 0);

    // Create a pull
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg, svm.latest_blockhash());
    let res = svm.send_transaction(tx).unwrap();
    println!("Create pull transaction result: {:#?}", res);

    let account = svm.get_account(&pull_pda).unwrap();
    let data = anchor_vote::Pull::try_deserialize(&mut account.data.as_slice());
    assert!(data.is_ok());

    let data = data.unwrap();
    assert_eq!(data.creator, user.pubkey());
    assert_eq!(data.candidate_count, 0);
    assert_eq!(data.bump, bump);
    assert_eq!(data.metadata.title, "Test Pull");
    assert_eq!(data.metadata.description, "This is a test pull");
}

#[test]
fn create_account_short_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");
    let (pull_pda, _) = create_pull(&mut svm, &user, "Test Pull", "This is another test pull", 0);

    let account = svm.get_account(&pull_pda).unwrap();
    let data = anchor_vote::Pull::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(data.creator, user.pubkey());
    assert_eq!(data.metadata.title, "Test Pull");
    assert_eq!(data.metadata.description, "This is another test pull");
}

#[test]
fn create_candidate_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");
    let (pull_pda, _) = create_pull(&mut svm, &user, "Test Pull", "This is a test pull", 0);

    let (ix, candidate_pda, bump) = ix_create_candidate(user.pubkey(), pull_pda.clone(), "Candidate 1", 0);

    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg, svm.latest_blockhash());
    svm.send_transaction(tx).unwrap();

    let candidate_account = svm.get_account(&candidate_pda).unwrap();
    let candidate_data = anchor_vote::Candidate::try_deserialize(&mut candidate_account.data.as_slice()).unwrap();

    assert_eq!(candidate_data.pull_key, pull_pda);
    assert_eq!(candidate_data.bump, bump);
    assert_eq!(candidate_data.name, "Candidate 1");
    assert_eq!(candidate_data.number_of_votes, 0);

    let pull_account = svm.get_account(&pull_pda).unwrap();
    let pull_data = anchor_vote::Pull::try_deserialize(&mut pull_account.data.as_slice()).unwrap();

    assert_eq!(pull_data.candidate_count, 1);
}

#[test]
fn create_couple_candidaete_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");
    let (pull_pda, _) = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let (candidate_1, _) = create_candidate(&mut svm, &user, pull_pda.clone(), "Rust", 0);
    // let (candidate_1, _) = create_candidate(&mut svm, &user, pull_pda.clone(), "Rust"); // what if same name, huh?
    let (candidate_2, _) = create_candidate(&mut svm, &user, pull_pda.clone(), "Python", 1);
    let (candidate_3, _) = create_candidate(&mut svm, &user, pull_pda.clone(), "Ruby", 2);
    let (candidate_4, _) = create_candidate(&mut svm, &user, pull_pda.clone(), "HTML FUCK YEAH", 3);

    assert!(svm.get_account(&candidate_1).is_some());
    assert!(svm.get_account(&candidate_2).is_some());
    assert!(svm.get_account(&candidate_3).is_some());
    assert!(svm.get_account(&candidate_4).is_some());

    let pull_account = svm.get_account(&pull_pda).unwrap();
    let pull_data = anchor_vote::Pull::try_deserialize(&mut pull_account.data.as_slice()).unwrap();
    assert_eq!(pull_data.candidate_count, 4);
}

#[test]
#[should_panic]
fn create_candidate_from_hacker_test() {
    let hacker = Keypair::new();

    let (mut svm, user) = init_svm_env("anchor_vote");
    let (pull_pda, _) = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let _ = create_candidate(&mut svm, &hacker, pull_pda.clone(), "Rust", 0); // not allowed!
}

#[test]
fn voting_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");
    set_svm_time(&mut svm, current_time() + 100);

    let user2 = create_user(&mut svm);
    let user3 = create_user(&mut svm);

    let (pull_pda, _) = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let (candidate_1, _) = create_candidate(&mut svm, &user, pull_pda.clone(), "Rust", 0);
    let (candidate_2, _) = create_candidate(&mut svm, &user, pull_pda.clone(), "Python", 1);

    let voter_trackers = [
        (voting(&mut svm, &user, pull_pda.clone(), candidate_1.clone()), candidate_1),
        (voting(&mut svm, &user2, pull_pda.clone(), candidate_1.clone()), candidate_1),
        (voting(&mut svm, &user3, pull_pda.clone(), candidate_2.clone()), candidate_2),
    ];

    for (voter_pda, candidate) in voter_trackers.iter() {
        let account = svm.get_account(voter_pda);
        assert!(account.is_some());
        let data = anchor_vote::Voter::try_deserialize(&mut account.unwrap().data.as_slice()).unwrap();
        assert_eq!(data.voted_for, *candidate);
    }

    let candidate_account = svm.get_account(&candidate_1).unwrap();
    let candidate_data = anchor_vote::Candidate::try_deserialize(&mut candidate_account.data.as_slice()).unwrap(); // todo: improve it, move to common
    assert_eq!(candidate_data.number_of_votes, 2);

    let candidate_account = svm.get_account(&candidate_2).unwrap();
    let candidate_data = anchor_vote::Candidate::try_deserialize(&mut candidate_account.data.as_slice()).unwrap(); // todo: improve it, move to common
    assert_eq!(candidate_data.number_of_votes, 1);

    let pull_account = svm.get_account(&pull_pda).unwrap();
    let pull_data = anchor_vote::Pull::try_deserialize(&mut pull_account.data.as_slice()).unwrap();
    assert_eq!(pull_data.candidate_count, 2);
}

#[test]
fn voting_with_price_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");

    let vote_price = LAMPORTS_PER_SOL / 2;
    let (pull_pda, _) = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", vote_price);
    let (candidate_1, _) = create_candidate(&mut svm, &user, pull_pda.clone(), "Rust", 0);

    // Voting started and voting for the candidate
    set_svm_time(&mut svm, current_time() + 100);
    let voter_pda = voting(&mut svm, &user, pull_pda.clone(), candidate_1.clone());

    // Now check the balance of the user
    let voter_account = svm.get_account(&voter_pda).unwrap();
    let rent = svm.get_sysvar::<solana_sdk::rent::Rent>();
    let expected_rent = rent.minimum_balance(8 + anchor_vote::Voter::INIT_SPACE);
    assert_eq!(voter_account.lamports, expected_rent + vote_price);
}
// todo: check closing shit and balance

#[test]
fn double_voting_test() {
    let (mut svm, user) = init_svm_env("anchor_vote");
    set_svm_time(&mut svm, current_time() + 100);

    let (pull, _) = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let (candidate, _) = create_candidate(&mut svm, &user, pull.clone(), "Rust", 0);

    let (ix, _) = ix_voting(user.pubkey(), pull, candidate);

    // First vote
    let msg = Message::new(&[ix], Some(&user.pubkey()));
    let tx = Transaction::new(&[&user], msg.clone(), svm.latest_blockhash());
    let result = svm.send_transaction(tx);
    assert!(result.is_ok());

    // Should return error, because user already voted in this pull
    let tx = Transaction::new(&[&user], msg.clone(), svm.latest_blockhash());
    let result = svm.send_transaction(tx);
    assert!(result.is_err());

    let candidate_account = svm.get_account(&candidate).unwrap();
    let candidate_data = anchor_vote::Candidate::try_deserialize(&mut candidate_account.data.as_slice()).unwrap();
    assert_eq!(candidate_data.number_of_votes, 1);
}
