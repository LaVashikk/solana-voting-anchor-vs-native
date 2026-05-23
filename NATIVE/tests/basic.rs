use dummy_sdk::prelude::*;
use native_voter_cheap::{instructions::create_vote::VotingArgs, state::{candidate::Candidate, pull::Pull, voter::Voter}};
use solana_sdk::{message::Message, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

mod common;
use common::*;

#[test]
fn create_account_test() {
    let (mut svm, user) = init_svm_env("native_voter_cheap");
    let pull_pubkey = create_pull(&mut svm, &user, "Test Pull", "This is another test pull", 0);

    let account = svm.get_account(&pull_pubkey).unwrap();
    let data = Pull::try_from_bytes(account.data.as_slice()).unwrap();
    assert_eq!(data.creator, user.pubkey());
    assert_eq!(data.title.as_str_lossy(), "Test Pull");
    assert_eq!(data.description.as_str_lossy(), "This is another test pull");
}


#[test]
fn create_candidate_test() {
    let (mut svm, user) = init_svm_env("native_voter_cheap");
    let pull_key = create_pull(&mut svm, &user, "Test Pull", "This is a test pull", 0);

    let candidate_key = create_candidate(&mut svm, &user, pull_key.clone(), "Candidate 1");

    let candidate_account = svm.get_account(&candidate_key).unwrap();
    let candidate_data = Candidate::try_from_bytes(candidate_account.data.as_slice()).unwrap();

    assert_eq!(candidate_data.pull, pull_key);
    assert_eq!(candidate_data.name.as_str_lossy(), "Candidate 1");
    assert_eq!(candidate_data.number_of_votes, 0);

    let pull_account = svm.get_account(&pull_key).unwrap();
    let pull_data = Pull::try_from_bytes(pull_account.data.as_slice()).unwrap();

    assert_eq!(pull_data.candidate_count, 1);
}

#[test]
fn create_couple_candidaete_test() {
    let (mut svm, user) = init_svm_env("native_voter_cheap");
    let pull_pda = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let candidate_1 = create_candidate(&mut svm, &user, pull_pda.clone(), "Rust");
    let candidate_2 = create_candidate(&mut svm, &user, pull_pda.clone(), "Python");
    let candidate_3 = create_candidate(&mut svm, &user, pull_pda.clone(), "Ruby");
    let candidate_4 = create_candidate(&mut svm, &user, pull_pda.clone(), "HTML FUCK YEAH");

    assert!(svm.get_account(&candidate_1).is_some());
    assert!(svm.get_account(&candidate_2).is_some());
    assert!(svm.get_account(&candidate_3).is_some());
    assert!(svm.get_account(&candidate_4).is_some());

    let mut previous: PodOption<Pubkey> = PodOption::none();
    for c in [&candidate_1, &candidate_2, &candidate_3, &candidate_4] {
        assert_eq!(
            Candidate::try_from_bytes( svm.get_account(c).unwrap().data.as_slice() ).unwrap().prev_candidate,
            previous
        );
        previous = PodOption::some(c.clone());
    }

    let pull_account = svm.get_account(&pull_pda).unwrap();
    let pull_data = Pull::try_from_bytes(pull_account.data.as_slice()).unwrap();
    assert_eq!(pull_data.candidate_count, 4);
    assert_eq!(pull_data.last_candidate, PodOption::some(candidate_4));
}

#[test]
#[should_panic]
fn create_candidate_from_hacker_test() {
    let hacker = Keypair::new();

    let (mut svm, user) = init_svm_env("native_voter_cheap");
    let pull = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let _ = create_candidate(&mut svm, &hacker, pull.clone(), "Rust"); // not allowed!
}

#[test]
fn voting_test() {
    let (mut svm, user) = init_svm_env("native_voter_cheap");
    set_svm_time(&mut svm, current_time() + 100);

    let user2 = create_user(&mut svm);
    let user3 = create_user(&mut svm);

    let pull_pda = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let candidate_1 = create_candidate(&mut svm, &user, pull_pda.clone(), "Rust");
    let candidate_2 = create_candidate(&mut svm, &user, pull_pda.clone(), "Python");

    let voter_trackers = [
        (create_vote(&mut svm, &user, pull_pda.clone(), candidate_1.clone()), candidate_1),
        (create_vote(&mut svm, &user2, pull_pda.clone(), candidate_1.clone()), candidate_1),
        (create_vote(&mut svm, &user3, pull_pda.clone(), candidate_2.clone()), candidate_2),
    ];

    for (voter_pda, candidate) in voter_trackers.iter() {
        let account = svm.get_account(voter_pda).unwrap();
        let data = Voter::try_from_bytes(account.data.as_slice()).unwrap();
        assert_eq!(data.voted_for, *candidate);
    }

    let candidate_account = svm.get_account(&candidate_1).unwrap();
    let candidate_data = Candidate::try_from_bytes(candidate_account.data.as_slice()).unwrap(); // todo: improve it, move to common
    assert_eq!(candidate_data.number_of_votes, 2);

    let candidate_account = svm.get_account(&candidate_2).unwrap();
    let candidate_data = Candidate::try_from_bytes(candidate_account.data.as_slice()).unwrap(); // todo: improve it, move to common
    assert_eq!(candidate_data.number_of_votes, 1);

    let pull_account = svm.get_account(&pull_pda).unwrap();
    let pull_data = Pull::try_from_bytes(pull_account.data.as_slice()).unwrap();
    assert_eq!(pull_data.candidate_count, 2);
}

#[test]
fn voting_with_price_test() {
    let (mut svm, user) = init_svm_env("native_voter_cheap");

    let vote_price = LAMPORTS_PER_SOL / 2;
    let pull = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", vote_price);
    let candidate_1 = create_candidate(&mut svm, &user, pull.clone(), "Rust");

    // Voting started and voting for the candidate
    set_svm_time(&mut svm, current_time() + 100);
    let voter_pda = create_vote(&mut svm, &user, pull.clone(), candidate_1.clone());

    // Now check the balance of the user
    let voter_account = svm.get_account(&voter_pda).unwrap();
    let rent = svm.get_sysvar::<solana_sdk::rent::Rent>();
    let expected_rent = rent.minimum_balance(Voter::SIZE);
    assert_eq!(voter_account.lamports, expected_rent + vote_price);
}
// todo: check closing shit and balance

#[test]
fn double_voting_test() {
    let (mut svm, user) = init_svm_env("native_voter_cheap");
    set_svm_time(&mut svm, current_time() + 100);

    let pull = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let candidate = create_candidate(&mut svm, &user, pull.clone(), "Rust");

    let (voter_pda, _) = Pubkey::find_program_address(&Voter::get_seeds(&pull, &user.pubkey()), &common::PROGRAM_ID);
    let accounts = native_voter_cheap::instructions::create_vote::client::VotingAccounts {
        voter: user.pubkey(),
        pull,
        candidate,
        voter_pda,
    };

    let ix = VotingArgs.build_ix(common::PROGRAM_ID, accounts);

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
    let candidate_data = Candidate::try_from_bytes(&mut candidate_account.data.as_slice()).unwrap();
    assert_eq!(candidate_data.number_of_votes, 1);
}
