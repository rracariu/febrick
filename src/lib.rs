// Copyright (c) 2024, Radu Racariu.

//!
//! FeBrick is a Rust crate that provides Rust and WebAssembly interface for the Brick Schema.
//!

pub mod brick;
pub mod property;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
