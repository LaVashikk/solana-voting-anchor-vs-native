mod common;
use common::*;
use dummy_sdk::accounts::pod::PodAccountData;
use litesvm::LiteSVM;
use native_voter_cheap::state::pull::Pull;
use solana_sdk::{message::Message, signer::Signer, transaction::Transaction};

#[test]
fn test_close_pull() {
    let (mut svm, creator) = init_svm_env("native_voter_cheap");
    let pull_pda = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);

    let creator_balance_before = svm.get_balance(&creator.pubkey()).unwrap();
    let expected_creator_share = svm.get_account(&pull_pda).unwrap().lamports;

    set_svm_time(&mut svm, current_time() + 100_000);

    close_pull(&mut svm, &creator, pull_pda.clone());

    // Account should be removed from the network
    assert!(svm.get_account(&pull_pda).is_none());

    // Creator should receive their share
    let creator_balance_after = svm.get_balance(&creator.pubkey()).unwrap();
    assert_eq!(creator_balance_after, creator_balance_before + expected_creator_share - 5000);
}

#[test]
fn test_close_not_ended_pull() {
    let (mut svm, creator) = init_svm_env("native_voter_cheap");
    set_svm_time(&mut svm, current_time());

    let pull_pda = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);
    create_candidate(&mut svm, &creator, pull_pda, "Candidate 1");

    let res = close_pull_raw(&mut svm, &creator, pull_pda.clone());

    let err = res.unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("Candidates not closed")), // TODO: for anchor, it's `CandidatesNotClosed`, but here - it text-based
        "Expected log 'CandidatesNotClosed', but got logs: {:#?}",
        err.meta.logs
    );
}

#[test]
fn test_close_invalid_pull() {
    let (mut svm, creator) = init_svm_env("native_voter_cheap");

    let pull_pda = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);
    let err = close_pull_raw(&mut svm, &creator, pull_pda.clone()).unwrap_err();

    assert!(
        err.meta.logs.iter().any(|l| l.contains("Voting not ended yet")), // TODO: for anchor, it's `VotingNotEnded`, but here - it text-based
        "Expected log 'VotingNotEnded', but got logs: {:#?}",
        err.meta.logs
    );

}

#[test]
fn test_close_candidate() {
    let (mut svm, creator) = init_svm_env("native_voter_cheap");

    let pull_pda = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);
    let candidates = [
        create_candidate(&mut svm, &creator, pull_pda, "Candidate 1"),
        create_candidate(&mut svm, &creator, pull_pda, "Candidate 2"),
    ];

    let get_candidate_count = |svm: &LiteSVM| -> u64 {
        let account = svm.get_account(&pull_pda).unwrap();
        let data = Pull::try_from_bytes(account.data.as_slice()).unwrap();
        data.candidate_count
    };

    assert_eq!(get_candidate_count(&svm), 2);
    set_svm_time(&mut svm, current_time() + 100_000);

    let creator_balance_before = svm.get_balance(&creator.pubkey()).unwrap();
    // dbg!(native_get_all_candidate(&svm, &pull_pda));

    for (i, candidate) in candidates.iter().rev().enumerate() {
        close_candidate(&mut svm, &creator, pull_pda, candidate.clone());

        let expected_count = (candidates.len() - i - 1) as u64;
        assert!(svm.get_account(&candidate).is_none());
        assert_eq!(get_candidate_count(&svm), expected_count);
    }

    assert_eq!(get_candidate_count(&svm), 0);
    // dbg!(native_get_all_candidate(&svm, &pull_pda));

    // Creator should receive their share
    let creator_balance_after = svm.get_balance(&creator.pubkey()).unwrap();
    assert!(creator_balance_after > creator_balance_before);
}

#[test]
fn test_close_voting_tracker() {
    let (mut svm, creator) = init_svm_env("native_voter_cheap");
    // Voting started
    set_svm_time(&mut svm, current_time() + 100);

    let pull_pda = create_pull(&mut svm, &creator, "BEST Portal 2 Mod", "Desc", 0);
    let candidate = create_candidate(&mut svm, &creator, pull_pda, "Project Capture");

    let voter_pda = create_vote(&mut svm, &creator, pull_pda, candidate);

    // Now voting is ended
    set_svm_time(&mut svm, current_time() + 100_000);
    close_vote(&mut svm, &creator, creator.pubkey(), voter_pda.clone());

    // Account should be removed from the network
    assert!(svm.get_account(&voter_pda).is_none());
}

#[test]
fn test_close_integrate() {
    let (mut svm, creator) = init_svm_env("native_voter_cheap");

    let pull_pda = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);
    let candidates = [
        create_candidate(&mut svm, &creator, pull_pda, "Candidate 1"),
        create_candidate(&mut svm, &creator, pull_pda, "Candidate 2"),
    ];

    let get_candidate_count = |svm: &LiteSVM| -> u64 {
        let account = svm.get_account(&pull_pda).unwrap();
        let data = Pull::try_from_bytes(account.data.as_slice()).unwrap();
        data.candidate_count
    };

    assert_eq!(get_candidate_count(&svm), 2);
    set_svm_time(&mut svm, current_time() + 100_000);

    let err = close_pull_raw(&mut svm, &creator, pull_pda.clone()).unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("Candidates not closed")), // TODO: for anchor, it's `CandidatesNotClosed`, but here - it text-based
        "Expected log 'CandidatesNotClosed', but got logs: {:#?}",
        err.meta.logs
    );

    let creator_balance_before = svm.get_balance(&creator.pubkey()).unwrap();
    svm.expire_blockhash();

    for (i, candidate) in candidates.iter().rev().enumerate() {
        close_candidate(&mut svm, &creator, pull_pda, candidate.clone());

        let expected_count = (candidates.len() - i - 1) as u64;
        assert!(svm.get_account(&candidate).is_none());
        assert_eq!(get_candidate_count(&svm), expected_count);
    }

    assert_eq!(get_candidate_count(&svm), 0);

    // Creator should receive their share
    let creator_balance_after = svm.get_balance(&creator.pubkey()).unwrap();
    assert!(creator_balance_after > creator_balance_before);

    assert_eq!(native_get_all_candidate(&svm, &pull_pda).len(), 0);
    close_pull(&mut svm, &creator, pull_pda.clone())
}
