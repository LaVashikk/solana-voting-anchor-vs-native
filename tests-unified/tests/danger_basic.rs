use tests_unified::*;

#[test]
fn test_cross_poll_voting() {
    let (mut svm, user) = init_svm_env(program_name());

    let pull_1 = create_pull(&mut svm, &user, "Poll 1", "Desc 1", 0);
    let pull_2 = create_pull(&mut svm, &user, "Poll 2", "Desc 2", 0);

    // Candidate belongs to poll_2
    let candidate_2 = create_candidate(&mut svm, &user, pull_2, "Python");

    // Try to vote in poll_1 with candidate from poll_2
    let res = create_vote_raw(&mut svm, &user, pull_1, candidate_2);

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("Invalid pull") || l.contains("InvalidPull")),
        "Expected log 'Invalid pull', but got logs: {:#?}",
        err.meta.logs
    );
}

#[test]
fn test_create_candidate_invalid_creator() {
    let (mut svm, creator) = init_svm_env(program_name());
    let other_user = create_user(&mut svm);

    let pull_pda = create_pull(&mut svm, &creator, "Best language", "Test", 0);

    // Try to create candidate by non-creator
    let res = create_candidate_raw(&mut svm, &other_user, pull_pda, "Rust");

    let err = res.1.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("Invalid creator") || l.contains("InvalidCreator")),
        "Expected log 'Invalid creator', but got logs: {:#?}",
        err.meta.logs
    );
}

#[test]
fn test_voting_after_end() {
    let (mut svm, creator) = init_svm_env(program_name());
    let user = create_user(&mut svm);

    let pull_pda = create_pull(&mut svm, &creator, "Best language", "Test", 0);
    let candidate = create_candidate(&mut svm, &creator, pull_pda, "Rust");

    // Move time forward past end (default end is current_time + 10_000 in create_pull)
    set_svm_time(&mut svm, current_time() + 20_000);

    let res = create_vote_raw(&mut svm, &user, pull_pda, candidate);

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("Voting Already Ended") || l.contains("VotingAlreadyEnded")),
        "Expected log 'Voting Already Ended', but got logs: {:#?}",
        err.meta.logs
    );
}

#[test]
fn test_voting_before_start() {
    let (mut svm, creator) = init_svm_env(program_name());
    let user = create_user(&mut svm);

    let now = current_time();
    let pull_pda = create_pull(&mut svm, &creator, "Best language", "Test", 0);
    let candidate = create_candidate(&mut svm, &creator, pull_pda, "Rust");

    // Set time to before pull creation (which used 'now' as start time)
    set_svm_time(&mut svm, now - 100);

    let res = create_vote_raw(&mut svm, &user, pull_pda, candidate);

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("Not started") || l.contains("NotStarted")),
        "Expected log 'Not started', but got logs: {:#?}",
        err.meta.logs
    );
}

#[test]
fn test_duplicate_vote() {
    let (mut svm, creator) = init_svm_env(program_name());
    let user = create_user(&mut svm);

    let pull_pda = create_pull(&mut svm, &creator, "Best language", "Test", 0);
    let candidate = create_candidate(&mut svm, &creator, pull_pda, "Rust");

    set_svm_time(&mut svm, current_time() + 10_000);

    // First vote
    create_vote(&mut svm, &user, pull_pda, candidate);

    // Second vote should fail (PDA already exists)
    svm.expire_blockhash();
    let res = create_vote_raw(&mut svm, &user, pull_pda, candidate);

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("already initialized") || l.contains("already in use") || l.contains("AlreadyInUse")),
        "Expected log about account already in use, but got logs: {:#?}",
        err.meta.logs
    );
}
