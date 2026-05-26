use tests_unified::*;

#[derive(Default)]
struct BenchResults {
    create_pull: u64,
    create_candidate: u64,
    voting: u64,
    close_candidate: u64,
    close_pull: u64,
}

#[test]
fn versus_benchmark() {
    let mut results = BenchResults::default();
    let program = program_name();
    let (mut svm, user) = init_svm_env(program);

    // Create Pull
    let (pull_pubkey, res) = create_pull_raw(&mut svm, &user, "Versus Bench", "Comparison of CU", 1000);
    results.create_pull = res.unwrap().compute_units_consumed;

    // Create Candidate
    let (candidate_pubkey, res) = create_candidate_raw(&mut svm, &user, pull_pubkey, "Rust");
    results.create_candidate = res.unwrap().compute_units_consumed;

    // Voting (using fixed keys for deterministic PDA CU)
    set_svm_time(&mut svm, current_time() + 100);
    let res = create_vote_raw(&mut svm, &user, pull_pubkey, candidate_pubkey);
    results.voting = res.unwrap().compute_units_consumed;

    // Close Candidate
    set_svm_time(&mut svm, current_time() + 100_000);
    let res = close_candidate_raw(&mut svm, &user, pull_pubkey, candidate_pubkey);
    results.close_candidate = res.unwrap().compute_units_consumed;

    // Close Pull
    let res = close_pull_raw(&mut svm, &user, pull_pubkey);
    results.close_pull = res.unwrap().compute_units_consumed;

    print_table(program, results);
}

fn print_table(name: &str, r: BenchResults) {
    println!("\nCU BENCHMARK RESULTS [{}]", name);
    println!("+------------------+----------------+");
    println!("| Instruction      | Compute Units  |");
    println!("+------------------+----------------+");
    println!("| Create Pull      | {:>14} |", r.create_pull);
    println!("| Create Candidate | {:>14} |", r.create_candidate);
    println!("| Voting           | {:>14} |", r.voting);
    println!("| Close Candidate  | {:>14} |", r.close_candidate);
    println!("| Close Pull       | {:>14} |", r.close_pull);
    println!("+------------------+----------------+");
}
