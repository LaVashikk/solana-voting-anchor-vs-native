pub mod align8;
pub mod bool;
pub mod option;
pub mod string;
pub mod enumerate;

pub use bool::*;
pub use option::*;
pub use string::*;
pub use enumerate::*;

// EXAMPLE OF LOGS:
// "Program log: Candidate { pull: GYUBxRYjWoUkGQbXrMui72FkP1YiTsSjFw7NJGvhuT1M, name: FixedString { data: \"Candidate 1\", len: 11 }, number_of_votes: 0, last_candidate: None }"
