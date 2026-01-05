use std::error::Error;
use std::fmt;

/// Custom error types for minacalc operations
#[derive(Debug)]
pub enum MinaCalcError {
    /// Calculator creation failed
    CalculatorCreationFailed,
    /// No notes provided for calculation
    NoNotesProvided,
    /// Invalid music rate (must be positive)
    InvalidMusicRate(f32),
    /// Invalid score goal (must be between 0 and 100)
    InvalidScoreGoal(f32),
    /// Calculation failed
    CalculationFailed(String),
    /// Invalid note data
    InvalidNoteData(String),
    /// Memory allocation failed
    MemoryAllocationFailed,
    /// Internal C++ error
    InternalError(String),
    /// ROX (rhythm-open-exchange) related error
    #[cfg(feature = "rox")]
    RoxError(RoxError),
    /// Unsupported key count (only 4K, 6K, 7K supported)
    UnsupportedKeyCount(u32),
}

/// Custom error types for ROX (rhythm-open-exchange) operations
#[cfg(feature = "rox")]
#[derive(Debug)]
pub enum RoxError {
    /// Failed to decode chart file
    DecodeFailed(String),
    /// Invalid rate value
    InvalidRate(f32),
    /// No notes in chart
    NoNotes,
    /// Invalid note data
    InvalidNote(String),
    /// Unsupported key count
    UnsupportedKeyCount(usize),
}

impl fmt::Display for MinaCalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MinaCalcError::CalculatorCreationFailed => write!(f, "Failed to create calculator"),
            MinaCalcError::NoNotesProvided => write!(f, "No notes provided for calculation"),
            MinaCalcError::InvalidMusicRate(rate) => {
                write!(f, "Invalid music rate: {} (must be positive)", rate)
            }
            MinaCalcError::InvalidScoreGoal(goal) => write!(
                f,
                "Invalid score goal: {} (must be between 0 and 100)",
                goal
            ),
            MinaCalcError::CalculationFailed(msg) => write!(f, "Calculation failed: {}", msg),
            MinaCalcError::InvalidNoteData(msg) => write!(f, "Invalid note data: {}", msg),
            MinaCalcError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            MinaCalcError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            #[cfg(feature = "rox")]
            MinaCalcError::RoxError(rox_err) => write!(f, "ROX error: {}", rox_err),
            MinaCalcError::UnsupportedKeyCount(count) => {
                write!(
                    f,
                    "Unsupported key count: {} (only 4K, 6K, 7K supported)",
                    count
                )
            }
        }
    }
}

#[cfg(feature = "rox")]
impl fmt::Display for RoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoxError::DecodeFailed(msg) => write!(f, "Failed to decode chart: {}", msg),
            RoxError::InvalidRate(rate) => write!(f, "Invalid rate: {} (must be positive)", rate),
            RoxError::NoNotes => write!(f, "No notes found in chart"),
            RoxError::InvalidNote(msg) => write!(f, "Invalid note: {}", msg),
            RoxError::UnsupportedKeyCount(count) => write!(f, "Unsupported key count: {}", count),
        }
    }
}

impl Error for MinaCalcError {}

#[cfg(feature = "rox")]
impl Error for RoxError {}

// Conversion from RoxError to MinaCalcError
#[cfg(feature = "rox")]
impl From<RoxError> for MinaCalcError {
    fn from(rox_err: RoxError) -> Self {
        MinaCalcError::RoxError(rox_err)
    }
}

// Type alias for common result types
pub type MinaCalcResult<T> = Result<T, MinaCalcError>;

#[cfg(feature = "rox")]
pub type RoxResult<T> = Result<T, RoxError>;
