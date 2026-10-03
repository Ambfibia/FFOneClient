//! Headless integration checks against the production character UI and localization.
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

#[path = "standalone_tests.rs"]
#[cfg(test)]
mod standalone_tests;
