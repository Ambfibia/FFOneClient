use std::{collections::BTreeMap, fs, path::Path};

use bevy::asset::{AssetApp, AssetPlugin};
use sha2::{Digest, Sha256};

use crate::localization::{Localization, LocalizationPlugin};
use crate::world_map::*;

mod constants;
mod operations;
mod codec;
mod animation;
mod containers;
mod systems;
mod assets;

use constants::EPSILON;
use operations::{assert_close, player, open_other_local};
use assets::{TestCatalog, catalog_entry};
