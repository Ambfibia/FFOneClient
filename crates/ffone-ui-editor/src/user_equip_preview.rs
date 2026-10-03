use ffone_ui_layout::{
    UiLayoutDocument, UiLayoutElement, UiLayoutForm, UiLayoutHorizontalAlign, UiLayoutImageMode,
    UiLayoutRect, UiLayoutSpace, UiLayoutText, UiLayoutVerticalAlign, UiLayoutVisual,
};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

mod constants;
mod operations_character_panel;
mod operations_nano_viewer;
mod state;
mod types;

use constants::{UI, FONT};
pub(super) use operations_character_panel::enrich;
use operations_nano_viewer::{
    nano_viewer, help_panel, spec, ensure, ensure_form, image, nine, image_text, text, dynamic,
    text_value, rect, white, cyan, yellow, green, dark_blue
};
use state::{inventory_window, inventory_content, character_status, item_mode_overview};
use types::{PopupPreviewVariant, ElementSpec, Align};
