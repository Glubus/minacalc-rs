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
    /// Osu! related error (deprecated, use RoxError)
    #[cfg(feature = "osu")]
    OsuError(OsuError),
    /// ROX (rhythm-open-exchange) related error
    #[cfg(feature = "rox")]
    RoxError(RoxError),
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

/// Custom error types for osu! beatmap operations (deprecated)
#[cfg(feature = "osu")]
#[derive(Debug)]
pub enum OsuError {
    /// Unsupported column position
    UnsupportedColumn(f32),
    /// Unsupported hit object kind
    UnsupportedHitObjectKind(String),
    /// Failed to convert hit object
    HitObjectConversion(String),
    /// Beatmap validation failed
    ValidationFailed(String),
    /// Failed to parse beatmap file
    ParseFailed(String),
    /// Unsupported game mode
    UnsupportedGameMode(String),
    /// Unsupported key count
    UnsupportedKeyCount(f32),
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
            #[cfg(feature = "osu")]
            MinaCalcError::OsuError(osu_err) => write!(f, "Osu! error: {}", osu_err),
            #[cfg(feature = "rox")]
            MinaCalcError::RoxError(rox_err) => write!(f, "ROX error: {}", rox_err),
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

#[cfg(feature = "osu")]
impl fmt::Display for OsuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OsuError::UnsupportedColumn(x) => write!(f, "Unsupported column position: {}", x),
            OsuError::UnsupportedHitObjectKind(kind) => {
                write!(f, "Unsupported hit object kind: {}", kind)
            }
            OsuError::HitObjectConversion(msg) => {
                write!(f, "Hit object conversion failed: {}", msg)
            }
            OsuError::ValidationFailed(msg) => write!(f, "Beatmap validation failed: {}", msg),
            OsuError::ParseFailed(msg) => write!(f, "Failed to parse beatmap: {}", msg),
            OsuError::UnsupportedGameMode(mode) => write!(f, "Unsupported game mode: {}", mode),
            OsuError::UnsupportedKeyCount(count) => write!(f, "Unsupported key count: {}", count),
        }
    }
}

impl Error for MinaCalcError {}

#[cfg(feature = "rox")]
impl Error for RoxError {}

#[cfg(feature = "osu")]
impl Error for OsuError {}

// Conversion from RoxError to MinaCalcError
#[cfg(feature = "rox")]
impl From<RoxError> for MinaCalcError {
    fn from(rox_err: RoxError) -> Self {
        MinaCalcError::RoxError(rox_err)
    }
}

// Conversion from OsuError to MinaCalcError
#[cfg(feature = "osu")]
impl From<OsuError> for MinaCalcError {
    fn from(osu_err: OsuError) -> Self {
        MinaCalcError::OsuError(osu_err)
    }
}

// Type alias for common result types
pub type MinaCalcResult<T> = Result<T, MinaCalcError>;

#[cfg(feature = "rox")]
pub type RoxResult<T> = Result<T, RoxError>;

#[cfg(feature = "osu")]
pub type OsuResult<T> = Result<T, OsuError>;
