use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// C++ calc allocation failed
    AllocationFailed,
    /// Notes slice was empty
    EmptyNotes,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AllocationFailed => write!(f, "failed to allocate calculator"),
            Error::EmptyNotes => write!(f, "notes slice is empty"),
        }
    }
}

impl std::error::Error for Error {}
