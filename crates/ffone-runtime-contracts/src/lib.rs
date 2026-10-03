//! Lightweight serialized contracts consumed by the native FFOne runtime.
//!
//! This crate deliberately contains no import, publication, filesystem mutation,
//! Unity recovery, or editor logic. Offline tools may produce these documents;
//! the game client only needs their stable schemas and data shapes.

#![forbid(unsafe_code)]

mod character_creation;
mod player_rig;
mod tutorial_effects;

pub use character_creation::*;
pub use player_rig::*;
pub use tutorial_effects::*;
