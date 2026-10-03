use crate::native_terrain::*;
use sha2::{Digest, Sha256};

mod assets;
mod operations;
mod terrain;
mod state;
mod codec;
mod textures;
mod materials;
mod models;
mod collision;
mod validation;
mod containers;

use assets::asset_root;
use operations::{remove_key_recursively, assert_vec3_exact};
use terrain::sanitize_runtime_terrain;
use state::sanitize_runtime_environment;
use collision::legacy_height_test_collider;
use validation::require_legacy_height;
