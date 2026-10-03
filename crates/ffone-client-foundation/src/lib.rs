//! Stable runtime foundation shared by the FFOne client shell.

#![forbid(unsafe_code)]

pub mod asset_tables;
pub mod assets;
pub mod coordinates;
pub mod native_packages;
pub mod semantic_audio;
pub mod xdt;

pub use native_packages::{NativePackageRegistry, PackageRegistryError, discover_native_packages};
