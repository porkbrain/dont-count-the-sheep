#![doc = include_str!("../README.md")]

#[cfg(feature = "poissons-eq")]
pub mod poissons_equation;
pub mod systems;
mod types;

pub use types::*;
