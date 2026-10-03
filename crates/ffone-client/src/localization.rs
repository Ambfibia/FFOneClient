//! Key-first EN/RU localization with deterministic English fallback.

mod images;

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

#[cfg(test)]
use std::fs;

use crate::{
    assets::{AssetLocator, LOCALIZATION_CATALOG_PATH},
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
};
use bevy::{
    prelude::*,
    text::{ComputedTextBlock, LineHeight, TextBrush, TextLayoutInfo},
    ui::UiSystems,
};

use serde::Deserialize;

#[cfg(test)]
#[path = "localization/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
mod tests;

mod localization;
mod types;
mod operations;
mod systems;
mod containers;
mod assets_native_path;
mod constants;

pub use localization::{
    DEFAULT_LANGUAGE, Localization, Language, VoiceLanguage, LocalizedVoice, LocalizedText,
    LocalizedTextCase, LocalizedTextLimit, LocalizationPlugin, LocalizationSet,
    localized_status, localized_tutorial_instruction, localized_tutorial_literal,
    localized_tutorial_mission_text, localized_tabledata_npc_name,
    localized_tabledata_npc_greeting, localized_tabledata_npc_barker,
    localized_tabledata_mission_barker, localized_tabledata_npc_skill_barker,
    localized_tabledata_quest_item_name, localized_world_location_text,
    localized_tutorial_scene_text
};
#[cfg(test)]
use localization::{
    apply_localized_texts, refresh_localized_voice_players, localized_voice_path, validate_locale,
    normalize_language, resolve_locale
};
pub use types::UiTextAutoFit;
use types::UiTextFitRegion;
use operations::{
    fit_region_height, fixed_text_region, admit_bounded_ui_text, auto_fit_ui_text,
    unique_fallback_source_keys, tutorial_instruction_key, template_args
};
#[cfg(test)]
use operations::{auto_fit_font_size, auto_fit_measurement};
pub use operations::semantic_key_for_source;
use systems::refresh_ui_text_fit_regions;
use containers::TextBundle;
use assets_native_path::native_path;
use constants::{TUTORIAL_INSTRUCTION_KEYS, REQUIRED_UI_KEYS};
