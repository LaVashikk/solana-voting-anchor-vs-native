use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};
use tests_unified::*;

#[test]
fn create_account_test() {
    let (mut svm, user) = init_svm_env(program_name());
    let pull_pubkey = create_pull(&mut svm, &user, "Test Pull", "This is another test pull", 0);

    let data = read_data::<Pull>(&svm, &pull_pubkey);
    assert_eq!(data.creator, user.pubkey());
    assert_eq!(data.title.as_str_lossy(), "Test Pull");
    assert_eq!(data.description.as_str_lossy(), "This is another test pull");
}


#[test]
fn create_candidate_test() {
    let (mut svm, user) = init_svm_env(program_name());
    let pull_key = create_pull(&mut svm, &user, "Test Pull", "This is a test pull", 0);

    let candidate_key = create_candidate(&mut svm, &user, pull_key.clone(), "Candidate 1");

    let candidate_data = read_data::<Candidate>(&svm, &candidate_key);

    assert_eq!(candidate_data.pull, pull_key);
    assert_eq!(candidate_data.name.as_str_lossy(), "Candidate 1");
    assert_eq!(candidate_data.number_of_votes, 0);

    let pull_data = read_data::<Pull>(&svm, &pull_key);

    assert_eq!(pull_data.candidate_count, 1);
}

#[test]
fn create_couple_candidaete_test() {
    let (mut svm, user) = init_svm_env(program_name());
    let pull_pda = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let candidate_1 = create_candidate(&mut svm, &user, pull_pda.clone(), "Rust");
    let candidate_2 = create_candidate(&mut svm, &user, pull_pda.clone(), "Python");
    let candidate_3 = create_candidate(&mut svm, &user, pull_pda.clone(), "Ruby");
    let candidate_4 = create_candidate(&mut svm, &user, pull_pda.clone(), "HTML FUCK YEAH");

    assert!(svm.get_account(&candidate_1).is_some());
    assert!(svm.get_account(&candidate_2).is_some());
    assert!(svm.get_account(&candidate_3).is_some());
    assert!(svm.get_account(&candidate_4).is_some());

    #[cfg(feature = "native")]
    {
        let mut previous = PodOption::none();
        for c in [&candidate_1, &candidate_2, &candidate_3, &candidate_4] {
            assert_eq!(
                read_data::<Candidate>(&svm, c).prev_candidate,
                previous
            );
            previous = PodOption::some(c.clone());
        }
    }

    let pull_data = read_data::<Pull>(&svm, &pull_pda);
    assert_eq!(pull_data.candidate_count, 4);

    #[cfg(feature = "native")]
    assert_eq!(pull_data.last_candidate, PodOption::some(candidate_4));
}

#[test]
#[should_panic]
fn create_candidate_from_hacker_test() {
    let hacker = Keypair::new();

    let (mut svm, user) = init_svm_env(program_name());
    let pull = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let _ = create_candidate(&mut svm, &hacker, pull.clone(), "Rust"); // not allowed!
}

#[test]
fn voting_test() {
    let (mut svm, user) = init_svm_env(program_name());
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
        let data = read_data::<Voter>(&svm, voter_pda);
        assert_eq!(data.voted_for, *candidate);
    }

    let candidate_data = read_data::<Candidate>(&svm, &candidate_1);
    assert_eq!(candidate_data.number_of_votes, 2);

    let candidate_data = read_data::<Candidate>(&svm, &candidate_2);
    assert_eq!(candidate_data.number_of_votes, 1);

    let pull_data = read_data::<Pull>(&svm, &pull_pda);
    assert_eq!(pull_data.candidate_count, 2);
}

#[test]
fn voting_with_price_test() {
    let (mut svm, user) = init_svm_env(program_name());

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
    assert!(voter_account.lamports >= expected_rent + vote_price);
}

#[test]
fn double_voting_test() {
    let (mut svm, user) = init_svm_env(program_name());
    set_svm_time(&mut svm, current_time() + 100);

    let pull = create_pull(&mut svm, &user, "Best programming language", "This is a test pull", 0);
    let candidate = create_candidate(&mut svm, &user, pull.clone(), "Rust");

    // First vote
    let res = create_vote_raw(&mut svm, &user, pull.clone(), candidate.clone());
    assert!(res.is_ok());

    // Should return error, because user already voted in this pull
    let res = create_vote_raw(&mut svm, &user, pull.clone(), candidate.clone());
    assert!(res.is_err());

    let candidate_data = read_data::<Candidate>(&svm, &candidate);
    assert_eq!(candidate_data.number_of_votes, 1);
}
