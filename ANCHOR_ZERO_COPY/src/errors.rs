use anchor_lang::error_code;

#[error_code]
pub enum VotingError {
    #[msg("Voting not ended yet")]
    VotingNotEnded,
    #[msg("Math error")]
    MathError,
    #[msg("Invalid creator")]
    InvalidCreator,
    #[msg("Invalid voter")]
    InvalidVoter,
    #[msg("Invalid pull")]
    InvalidPull,
    #[msg("Invalid candidate")]
    InvalidCandidate,

    #[msg("Candidates not closed")]
    CandidatesNotClosed,

    #[msg("Invalid Time Range")]
    InvalidTimeRange,
    #[msg("Voting Already Ended")]
    VotingAlreadyEnded,
    #[msg("Voting already started")]
    VotingAlreadyStarted,

    #[msg("Not started")]
    VotingNotStarted,

    #[msg("Name too long")]
    NameTooLong,
    #[msg("Description too long")]
    DescTooLong,
}
