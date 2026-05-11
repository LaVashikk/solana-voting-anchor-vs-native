#![allow(dead_code)]

use litesvm::LiteSVM;
use solana_sdk::{clock::Clock, native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};
use std::{env, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

mod ix_builder;
mod caller;

pub use ix_builder::*;
pub use caller::*;

pub fn current_time() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

pub fn create_user(svm: &mut LiteSVM) -> Keypair {
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), LAMPORTS_PER_SOL).unwrap();
    let balance = svm.get_balance(&user.pubkey()).unwrap();
    assert_eq!(balance, LAMPORTS_PER_SOL);
    user
}


pub fn init_svm_env(program: &str) -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();

    let target_path = env::var("TARGET_PATH").ok()
        .map(PathBuf::from)
        .or_else(|| {
            env::current_dir()
                .ok()
                .map(|cwd| cwd.join("target"))
                .filter(|dir| dir.is_dir())
        })
        .unwrap_or_else(|| {
            panic!("Cannot find 'target' directory, use \"TARGET_PATH\" environment variable to specify path to 'target' directory")
        });

    let program_path = target_path.join("deploy").join(program).with_extension("so");
    println!("Program path: {}", program_path.display());

    // let program_path = env::home_dir().unwrap().join(".cargo").join("target").join("deploy").join(program).with_extension("so"); // todo: only for local testing

    let program_bytes = std::fs::read(program_path)
        .expect("cargo build-sbf!");
    svm.add_program(anchor_vote::ID, &program_bytes).unwrap();

    let user = create_user(&mut svm);

    (svm, user)
}

pub fn set_svm_time(svm: &mut LiteSVM, time: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = time;
    svm.set_sysvar(&clock);

}
