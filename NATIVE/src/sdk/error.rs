use std::fmt;
use std::error::Error;

// todo: thiserror

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapacityError {
    pub max_capacity: usize,
    pub actual_length: usize,
}
impl Error for CapacityError {}
impl fmt::Display for CapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "String is too long: actual length is {} bytes, but capacity is {} bytes",
            self.actual_length, self.max_capacity
        )
    }
}
