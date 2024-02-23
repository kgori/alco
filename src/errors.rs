use std::error::Error;
use std::fmt;

// Define a custom error type
#[derive(Debug)]
pub struct AcError {
    pub(crate) message: String,
}

// Implement the Error trait for the custom error type
impl Error for AcError {}

// Implement the Display trait for the custom error type
impl fmt::Display for AcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}