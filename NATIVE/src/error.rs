use crate::{declare_error, define_program_error};

define_program_error! {
    pub enum VotingError {
        #[error("Voting not ended yet")]
        VotingNotEnded = 7000,
        #[error("Math error")]
        MathError,
        #[error("Invalid creator")]
        InvalidCreator,
        #[error("Invalid voter")]
        InvalidVoter,
        #[error("Invalid pull")]
        InvalidPull,

        #[error("Candidates not closed")]
        CandidatesNotClosed,

        #[error("Invalid Time Range")]
        InvalidTimeRange,
        #[error("Voting Already Ended")]
        VotingAlreadyEnded,
        #[error("Voting already started")]
        VotingAlreadyStarted,

        #[error("Not started")]
        VotingNotStarted,

        #[error("Name too long")]
        NameTooLong,
        #[error("Description too long")]
        DescTooLong,
    }
}

mod test {
    #[test]
    fn test_voting_error() {
        use super::VotingError;
        let err = VotingError::VotingNotEnded;
        assert_eq!(err.to_string(), "Voting not ended yet");
        assert_eq!(err as u32, 7000);

        let err = VotingError::try_from(7001).unwrap();
        assert_eq!(err.to_string(), "Math error");
        println!("{:?}", err);
    }
}
