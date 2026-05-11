// mod common;
// use common::*;
// use solana_sdk::{message::Message, signer::Signer, transaction::{Transaction, TransactionError}};

// #[test]
// fn test_exploit_close_pull_with_fake_creator() {
//     let (mut svm, real_creator) = init_svm_env("native_voter_cheap");

//     // Attacker creates their own user account
//     let attacker = create_user(&mut svm);

//     // Real creator initializes the pull
//     let (pull_pda, _) = create_pull(&mut svm, &real_creator, "Time to close", "Desc", 0);

//     set_svm_time(&mut svm, current_time() + 100_000);

//     // EXPLOIT ATTEMPT: The attacker tries to close the pull but passes THEIR OWN pubkey (todo)
//     // as the `creator` account. If the contract doesn't check `pull.creator == creator.key()`,
//     // the attacker will successfully steal the 95% rent refund intended for the real creator.
//     let ix = ix_close_pull(attacker.pubkey(), pull_pda.clone());

//     let msg = Message::new(&[ix], Some(&attacker.pubkey()));
//     let tx = Transaction::new(&[&attacker], msg, svm.latest_blockhash());
//     let res = svm.send_transaction(tx);

//     let err = res.unwrap_err();

//     // The transaction MUST fail. Anchor usually handles this via `#[account(has_one = creator)]`
//     // or seed constraints. We expect a Constraint error.
//     assert!(
//         err.meta.logs.iter().any(|l| l.contains("InvalidCreator")),
//         "The contract allowed a fake creator to close the pull! Logs: {:#?}",
//         err.meta.logs
//     );
// }

// #[test]
// fn test_exploit_close_candidate_mismatched_pull() {
//     let (mut svm, creator) = init_svm_env("native_voter_cheap");

//     // Create two completely separate pull accounts
//     let (pull_1_pda, _) = create_pull(&mut svm, &creator, "Pull 1", "Desc 1", 0);
//     let (pull_2_pda, _) = create_pull(&mut svm, &creator, "Pull 2", "Desc 2", 0);

//     // Create a candidate attached ONLY to Pull 1
//     let (candidate_pda, _) = create_candidate(&mut svm, &creator, pull_1_pda, "Candidate 1", 0);

//     // Fast-forward time
//     set_svm_time(&mut svm, current_time() + 100_000);

//     // EXPLOIT ATTEMPT: The bot tries to close Candidate 1, but passes Pull 2 as the context.
//     let ix = ix_close_candidate(creator.pubkey(), pull_2_pda.clone(), candidate_pda.clone());

//     let msg = Message::new(&[ix], Some(&creator.pubkey()));
//     let tx = Transaction::new(&[&creator], msg, svm.latest_blockhash());
//     let res = svm.send_transaction(tx);

//     let err = res.unwrap_err();

//     // The transaction MUST fail. We expect an Anchor constraint error (seed mismatch or has_one).
//     assert!(
//         err.meta.logs.iter().any(|l| l.contains("InvalidPull")),
//         "Contract allowed closing a candidate using the wrong Pull PDA! Logs: {:#?}",
//         err.meta.logs
//     );
// }

// #[test]
// fn test_exploit_double_close_candidate() {
//     let (mut svm, creator) = init_svm_env("native_voter_cheap");

//     let (pull_pda, _) = create_pull(&mut svm, &creator, "Pull", "Desc", 0);
//     let (candidate_pda, _) = create_candidate(&mut svm, &creator, pull_pda, "Candidate", 0);

//     // Fast-forward time
//     set_svm_time(&mut svm, current_time() + 100_000);

//     // LEGITIMATE ACTION: closes the candidate for the first time.
//     let ix1 = ix_close_candidate(creator.pubkey(), pull_pda.clone(), candidate_pda.clone());
//     let msg1 = Message::new(&[ix1], Some(&creator.pubkey()));
//     let tx1 = Transaction::new(&[&creator], msg1, svm.latest_blockhash());
//     svm.send_transaction(tx1).unwrap(); // Should succeed

//     // EXPLOIT ATTEMPT: tries to close the EXACT SAME candidate again.
//     let ix2 = ix_close_candidate(creator.pubkey(), pull_pda.clone(), candidate_pda.clone());
//     let msg2 = Message::new(&[ix2], Some(&creator.pubkey()));
//     let tx2 = Transaction::new(&[&creator], msg2, svm.latest_blockhash());
//     let res = svm.send_transaction(tx2);

//     let err = res.unwrap_err();
//     assert_eq!(err.err, TransactionError::AlreadyProcessed);
// }

// #[test]
// fn test_exploit_double_close_voting() {
//     let (mut svm, creator) = init_svm_env("native_voter_cheap");
//     let user = create_user(&mut svm);
//     let bot = create_user(&mut svm);

//     let (pull_pda, _) = create_pull(&mut svm, &creator, "Hack me", "Desc", 0);
//     let (candidate_pda, _) = create_candidate(&mut svm, &creator, pull_pda.clone(), "Target", 0);

//     // Start voting and vote for the candidate
//     set_svm_time(&mut svm, current_time() + 100);
//     let voter_pda = voting(&mut svm, &user, pull_pda.clone(), candidate_pda.clone());

//     // Now fast-forward time to close the voting
//     set_svm_time(&mut svm, current_time() + 100_000);

//     let bot_balance_before = svm.get_balance(&bot.pubkey()).unwrap();

//     // HACK ATTEMPT: Bot tries to close the voting twice in a row.
//     let ix1 = ix_close_voting(bot.pubkey(), creator.pubkey(), voter_pda.clone());
//     let ix2 = ix_close_voting(bot.pubkey(), creator.pubkey(), voter_pda.clone());

//     // put both instructions in one Message
//     let msg = Message::new(&[ix1, ix2], Some(&bot.pubkey()));
//     let tx = Transaction::new(&[&bot], msg, svm.latest_blockhash());

//     // Try to execute
//     let res = svm.send_transaction(tx);

//     // The transaction MUST fail! If it succeeds, the contract is vulnerable.
//     assert!(res.is_err(), "VULNERABILITY: Double close succeeded!");

//     // Since the transaction is atomic and rolled back entirely, the bot's balance should not change
//     let bot_balance_after = svm.get_balance(&bot.pubkey()).unwrap();
//     assert_eq!(
//         bot_balance_before, bot_balance_after + 5000,
//     );
// }
