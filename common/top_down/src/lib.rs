#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub mod actor;
pub mod layout;

pub use actor::{player::Player, Actor, ActorTarget};
pub use layout::{IntoMap, Map, SquareKind};
