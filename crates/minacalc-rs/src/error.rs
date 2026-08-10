use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// C++ calc allocation failed
    AllocationFailed,
    /// Notes slice was empty
    EmptyNotes,
    /// A custom-rate calculation received no rates.
    EmptyRates,
    /// A configuration field was outside its accepted domain.
    InvalidConfig(&'static str),
    /// A calculation argument was outside its accepted domain.
    InvalidArgument(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AllocationFailed => write!(f, "failed to allocate calculator"),
            Error::EmptyNotes => write!(f, "notes slice is empty"),
            Error::EmptyRates => write!(f, "rates slice is empty"),
            Error::InvalidConfig(message) => write!(f, "invalid calculator config: {message}"),
            Error::InvalidArgument(message) => write!(f, "invalid calculation argument: {message}"),
        }
    }
}

impl std::error::Error for Error {}
