mod common;
use common::*;

#[test]
fn test_exploit_close_pull_with_fake_creator() {
    let (mut svm, real_creator) = init_svm_env(if cfg!(feature = "anchor") { "anchor_vote" } else { "native_voter_cheap" });
    set_svm_time(&mut svm, current_time());

    // Attacker creates their own user account
    let attacker = create_user(&mut svm);

    // Real creator initializes the pull
    let pull_pda = create_pull(&mut svm, &real_creator, "Time to close", "Desc", 0);

    set_svm_time(&mut svm, current_time() + 100_000);

    // EXPLOIT ATTEMPT: The attacker tries to close the pull passing themselves as creator
    let res = close_pull_raw(&mut svm, &attacker, pull_pda);

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("Invalid creator") || l.contains("InvalidCreator")),
        "Expected 'Invalid creator' error, but got: {:#?}",
        err.meta.logs
    );
}

#[test]
fn test_exploit_close_candidate_mismatched_pull() {
    let (mut svm, creator) = init_svm_env(if cfg!(feature = "anchor") { "anchor_vote" } else { "native_voter_cheap" });
    set_svm_time(&mut svm, current_time());

    // Create two completely separate pull accounts
    let pull_1_pda = create_pull(&mut svm, &creator, "Pull 1", "Desc 1", 0);
    let pull_2_pda = create_pull(&mut svm, &creator, "Pull 2", "Desc 2", 0);

    // Create a candidate attached ONLY to Pull 1
    let candidate_pda = create_candidate(&mut svm, &creator, pull_1_pda, "Candidate 1");

    // Fast-forward time
    set_svm_time(&mut svm, current_time() + 100_000);

    // EXPLOIT ATTEMPT: Try to close Candidate 1, but passing Pull 2 as the context.
    let res = close_candidate_raw(&mut svm, &creator, pull_2_pda, candidate_pda);

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("Invalid pull") || l.contains("InvalidPull")),
        "Expected 'Invalid pull' error, but got: {:#?}",
        err.meta.logs
    );
}

#[test]
fn test_exploit_double_close_candidate() {
    let (mut svm, creator) = init_svm_env(if cfg!(feature = "anchor") { "anchor_vote" } else { "native_voter_cheap" });
    set_svm_time(&mut svm, current_time());

    let pull_pda = create_pull(&mut svm, &creator, "Pull", "Desc", 0);
    let candidate_pda = create_candidate(&mut svm, &creator, pull_pda, "Candidate");

    // Fast-forward time
    set_svm_time(&mut svm, current_time() + 100_000);

    // LEGITIMATE ACTION: closes the candidate for the first time.
    close_candidate(&mut svm, &creator, pull_pda, candidate_pda);

    // EXPLOIT ATTEMPT: tries to close the EXACT SAME candidate again.
    svm.expire_blockhash();
    let res = close_candidate_raw(&mut svm, &creator, pull_pda, candidate_pda);

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| 
            l.contains("owner is not allowed") || 
            l.contains("AccountNotFound") || 
            l.contains("IllegalOwner") ||
            l.contains("AccountNotInitialized")
        ),
        "Expected error for already closed account, but got: {:#?}",
        err.meta.logs
    );
}

#[test]
fn test_exploit_double_close_voting() {
    let (mut svm, creator) = init_svm_env(if cfg!(feature = "anchor") { "anchor_vote" } else { "native_voter_cheap" });
    set_svm_time(&mut svm, current_time());
    let user = create_user(&mut svm);
    let bot = create_user(&mut svm);

    let pull_pda = create_pull(&mut svm, &creator, "Hack me", "Desc", 0);
    let candidate_pda = create_candidate(&mut svm, &creator, pull_pda.clone(), "Target");

    // Start voting and vote for the candidate
    set_svm_time(&mut svm, current_time() + 100);
    let voter_pda = create_vote(&mut svm, &user, pull_pda.clone(), candidate_pda.clone());

    // Now fast-forward time to close the voting
    set_svm_time(&mut svm, current_time() + 100_000);

    let bot_balance_before = svm.get_balance(&bot.pubkey()).unwrap();

    let ix = ix_close_vote(bot.pubkey(), user.pubkey(), voter_pda);

    // put both instructions in one Message
    let msg = Message::new(&[ix.clone(), ix], Some(&bot.pubkey()));
    let tx = Transaction::new(&[&bot], msg, svm.latest_blockhash());

    // Try to execute
    let res = svm.send_transaction(tx);

    // The transaction MUST fail!
    assert!(res.is_err(), "VULNERABILITY: Double close succeeded!");
    let err = res.unwrap_err();

    // The second instruction should fail because the account was closed by the first one.
    assert!(
        err.meta.logs.iter().any(|l| 
            l.contains("owner is not allowed") || 
            l.contains("AccountNotFound") || 
            l.contains("IllegalOwner") ||
            l.contains("AccountNotInitialized")
        ),
        "Expected error for already closed account in second instruction, but got: {:#?}",
        err.meta.logs
    );

    // Since the transaction is atomic and rolled back entirely, the bot's balance should not change (except fee)
    let bot_balance_after = svm.get_balance(&bot.pubkey()).unwrap();
    assert_eq!(
        bot_balance_before, bot_balance_after + 5000,
    );
}
