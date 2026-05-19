// mod common;
// use anchor_lang::AccountDeserialize;
// use common::*;
// use litesvm::LiteSVM;
// use solana_sdk::{message::Message, signer::Signer, transaction::Transaction};

// #[test]
// fn test_close_pull() {
//     let (mut svm, creator) = init_svm_env("native_voter_cheap");
//     let (pull_pda, _) = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);

//     let creator_balance_before = svm.get_balance(&creator.pubkey()).unwrap();
//     let expected_creator_share = svm.get_account(&pull_pda).unwrap().lamports;

//     set_svm_time(&mut svm, current_time() + 100_000);

//     let ix = ix_close_pull(creator.pubkey(), pull_pda.clone());

//     let msg = Message::new(&[ix], Some(&creator.pubkey()));
//     let tx = Transaction::new(&[&creator], msg, svm.latest_blockhash());
//     svm.send_transaction(tx).unwrap();

//     // Account should be removed from the network
//     assert!(svm.get_account(&pull_pda).is_none());

//     // Creator should receive their share
//     let creator_balance_after = svm.get_balance(&creator.pubkey()).unwrap();
//     require_eq!(creator_balance_after, creator_balance_before + expected_creator_share - 5000);
// }

// #[test]
// fn test_close_not_ended_pull() {
//     let (mut svm, creator) = init_svm_env("native_voter_cheap");
//     set_svm_time(&mut svm, current_time());

//     let (pull_pda, _) = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);
//     create_candidate(&mut svm, &creator, pull_pda, "Candidate 1", 0);

//     let ix = ix_close_pull(creator.pubkey(), pull_pda.clone());

//     let msg = Message::new(&[ix], Some(&creator.pubkey()));
//     let tx = Transaction::new(&[&creator], msg, svm.latest_blockhash());
//     let res = svm.send_transaction(tx);

//     let err = res.unwrap_err();
//     assert!(
//         err.meta.logs.iter().any(|l| l.contains("CandidatesNotClosed")),
//         "Expected log 'CandidatesNotClosed', but got logs: {:#?}",
//         err.meta.logs
//     );
// }

// #[test]
// fn test_close_invalid_pull() {
//     let (mut svm, creator) = init_svm_env("native_voter_cheap");

//     let (pull_pda, _) = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);

//     let ix = ix_close_pull(creator.pubkey(), pull_pda.clone());

//     let msg = Message::new(&[ix], Some(&creator.pubkey()));
//     let tx = Transaction::new(&[&creator], msg, svm.latest_blockhash());
//     let err = svm.send_transaction(tx).unwrap_err();

//     assert!(
//         err.meta.logs.iter().any(|l| l.contains("VotingNotEnded")),
//         "Expected log 'VotingNotEnded', but got logs: {:#?}",
//         err.meta.logs
//     );

// }

// #[test]
// fn test_close_candidate() {
//     let (mut svm, creator) = init_svm_env("native_voter_cheap");

//     let (pull_pda, _) = create_pull(&mut svm, &creator, "Time to close", "Desc", 0);
//     let candidates = [
//         create_candidate(&mut svm, &creator, pull_pda, "Candidate 1", 0).0,
//         create_candidate(&mut svm, &creator, pull_pda, "Candidate 2", 1).0,
//     ];

//     let get_candidate_count = |svm: &LiteSVM| -> u64 {
//         let account = svm.get_account(&pull_pda).unwrap();
//         let data = native_voter_cheap::Pull::try_deserialize(&mut account.data.as_slice()).unwrap();
//         data.candidate_count
//     };

//     require_eq!(get_candidate_count(&svm), 2);
//     set_svm_time(&mut svm, current_time() + 100_000);

//     let creator_balance_before = svm.get_balance(&creator.pubkey()).unwrap();

//     for (i, candidate) in candidates.iter().enumerate() {
//         let ix = ix_close_candidate(creator.pubkey(), pull_pda.clone(), candidate.clone());
//         let msg = Message::new(&[ix], Some(&creator.pubkey()));
//         let tx = Transaction::new(&[&creator], msg, svm.latest_blockhash());

//         svm.send_transaction(tx).unwrap();

//         let expected_count = (candidates.len() - i - 1) as u64;
//         assert!(svm.get_account(&candidate).is_none());
//         require_eq!(get_candidate_count(&svm), expected_count);
//     }

//     require_eq!(get_candidate_count(&svm), 0);

//     // Creator should receive their share
//     let creator_balance_after = svm.get_balance(&creator.pubkey()).unwrap();
//     assert!(creator_balance_after > creator_balance_before);
// }

// #[test]
// fn test_close_voting_tracker() {
//     let (mut svm, creator) = init_svm_env("native_voter_cheap");
//     // Voting started
//     set_svm_time(&mut svm, current_time() + 100);

//     let (pull_pda, _) = create_pull(&mut svm, &creator, "BEST Portal 2 Mod", "Desc", 0);
//     let (candidate, _) = create_candidate(&mut svm, &creator, pull_pda, "Project Capture", 0);

//     let voter_pda = voting(&mut svm, &creator, pull_pda, candidate);

//     // Now voting is ended
//     set_svm_time(&mut svm, current_time() + 100_000);

//     // closing voter
//     let ix = ix_close_voting(creator.pubkey(), creator.pubkey(), voter_pda.clone());
//     let msg = Message::new(&[ix], Some(&creator.pubkey()));
//     let tx = Transaction::new(&[&creator], msg, svm.latest_blockhash());
//     svm.send_transaction(tx).unwrap();

//     // Account should be removed from the network
//     assert!(svm.get_account(&voter_pda).is_none());

// }
