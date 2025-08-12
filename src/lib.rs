//! Rust bindings for MinaCalc C++ library
//! 
//! This crate provides safe Rust bindings for the MinaCalc rhythm game difficulty calculator.

mod wrapper;

// Inclure les bindings générés automatiquement
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub use wrapper::*;
