//! Native EN/RU authoring. Drafts are merged by semantic key; files are replaced atomically.
use super::*;
#[path = "strings_tools.rs"]
pub(super) mod editing_tools;
#[path = "strings_exchange.rs"]
mod exchange;
use std::{
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "strings/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "strings/bilingual_tests.rs"]
mod bilingual_tests;

#[path = "strings/types.rs"]
mod types;
#[path = "strings/operations_buttons.rs"]
mod operations_buttons;
#[path = "strings/operations_draw.rs"]
mod operations_draw;
#[path = "strings/containers.rs"]
mod containers;
#[path = "strings/validation.rs"]
mod validation;
#[path = "strings/models.rs"]
mod models;
#[path = "strings/commands.rs"]
mod commands;

use types::{FileStamp, StringsRoot, StringCell, CellMetric, TableViewport, TableTrack};
pub(super) use types::{StringEditor, EditorSectionTitle, StringsPlugin};
use operations_buttons::{
    stamps, placeholders, merge, section_header, body_visibility, button, value, buttons,
    keyboard, persistence, measure_rows, estimated_height, visible_range, scroll_table
};
use operations_draw::draw;
use containers::read_bundle;
use validation::validate;
pub(super) use models::ModelEditorBody;
use commands::Action;
