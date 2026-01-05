//! Rust bindings for MinaCalc C++ library
//!
//! This crate provides safe Rust bindings for the MinaCalc rhythm game difficulty calculator.
//!
//! ## Features
//!
//! - `hashmap` (default): Provides HashMap conversion for MSD results
//! - `thread`: Provides thread-safe calculator pool
//! - `rox`: Provides universal rhythm game chart parsing (osu!, StepMania, etc.)

mod error;
mod wrapper;

// Include automatically generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Re-export wrapper types
pub use error::*;
pub use wrapper::*;

// Feature-gated modules
#[cfg(feature = "hashmap")]
pub mod hashmap;

#[cfg(feature = "thread")]
pub mod thread;

#[cfg(feature = "rox")]
pub mod rox;

#[cfg(feature = "utils")]
pub mod utils;

// Re-export feature-gated modules
#[cfg(feature = "hashmap")]
pub use hashmap::*;

#[cfg(feature = "thread")]
pub use thread::*;

#[cfg(feature = "rox")]
pub use rox::*;

#[cfg(feature = "utils")]
pub use utils::*;
