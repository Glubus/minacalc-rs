//! Rust bindings for MinaCalc C++ library
//! 
//! This crate provides safe Rust bindings for the MinaCalc rhythm game difficulty calculator.

mod wrapper;

// Include automatically generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub use wrapper::*;
