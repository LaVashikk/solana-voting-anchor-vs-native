#![allow(dead_code, unused)]
pub use solana_sdk::{
    clock::Clock,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    message::Message,
    transaction::Transaction,
};
pub use litesvm::LiteSVM;
use std::{env, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

pub const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("9AvUNHjxscdkiKQ8tUn12QCMXtcnbR9BVGq3ULNzFMRi");
pub const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");

#[cfg(not(any(
    feature = "anchor",
    feature = "anchor-zero-copy",
    feature = "native"
)))]
compile_error!("At least one of the following features must be enabled: `anchor`, `anchor-zero-copy`, or `native`.");

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

pub const fn program_name() -> &'static str {
    if cfg!(feature = "anchor") {
        "anchor_vote"
    } else if cfg!(feature = "anchor-zero-copy") {
        "zero_copy_anchor_vote"
    } else {
        "native_voter_cheap"
    }
}

pub fn get_target_path() -> PathBuf {
    env::var("TARGET_PATH").ok()
        .map(PathBuf::from)
        .or_else(|| {
            env::current_dir()
                .ok()
                .map(|cwd| cwd.join("target").join("deploy"))
                .filter(|dir| dir.is_dir())
        })
        .unwrap_or_else(|| {
             PathBuf::from("target/deploy")
        })

    // env::home_dir().unwrap().join(".cargo").join("target").join("deploy") // todo: only for local testing
}
pub fn get_program_path(program: &str) -> PathBuf {
    let target_path = get_target_path();
    target_path.join(program).with_extension("so")
}

pub fn init_svm_env(program: &str) -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();

    let program_path = get_program_path(program);
    let program_bytes = std::fs::read(&program_path)
        .unwrap_or_else(|_| panic!("Failed to read program at {}. Did you run cargo build-sbf?", program_path.display()));


    svm.add_program(PROGRAM_ID, &program_bytes).unwrap();

    let user = create_user(&mut svm);

    (svm, user)
}

pub fn set_svm_time(svm: &mut LiteSVM, time: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = time;
    svm.set_sysvar(&clock);
}

// --- Shared Wrapper Types ---

#[derive(Debug, PartialEq, Clone)]
pub struct UnifiedString(pub String);
impl UnifiedString {
    pub fn as_str_lossy(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PodOption<T>(pub Option<T>);
impl<T> PodOption<T> {
    pub const fn none() -> Self { Self(None) }
    pub const fn some(val: T) -> Self { Self(Some(val)) }
    pub fn is_some(&self) -> bool { self.0.is_some() }
    pub fn is_none(&self) -> bool { self.0.is_none() }
    pub fn unwrap(self) -> T { self.0.unwrap() }
}

pub trait UnifiedState: Sized {
    fn try_from_bytes(data: &[u8]) -> Result<Self, anyhow::Error>;
}

pub fn read_data<T: UnifiedState>(svm: &LiteSVM, pubkey: &Pubkey) -> T {
    let account = svm.get_account(pubkey).expect("Account not found");
    T::try_from_bytes(&account.data).expect("Failed to deserialize account data")
}

// --- Module Routing ---
// How test or write it in IDE? Just set one of the features as default in cargo.toml :)

#[cfg(feature = "native")]
pub mod native;
#[cfg(feature = "native")]
pub use native::*;

#[cfg(feature = "anchor")]
pub mod anchor;
#[cfg(feature = "anchor")]
pub use anchor::*;

#[cfg(feature = "anchor-zero-copy")]
pub mod anchor_zero_copy;
#[cfg(feature = "anchor-zero-copy")]
pub use anchor_zero_copy::*;
