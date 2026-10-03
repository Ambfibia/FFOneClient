mod user_equip_preview;

use eframe::egui::{
    self, Color32, FontFamily, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2,
};
use ffone_ui_layout::{
    UiLayoutDocument, UiLayoutForm, UiLayoutHorizontalAlign, UiLayoutImage, UiLayoutImageMode,
    UiLayoutRect, UiLayoutVerticalAlign, UiLayoutVisual,
};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> eframe::Result<()> {
    let (layout_path, background_path) = arguments();
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("FFOne UI Editor")
            .with_inner_size([1600.0, 950.0])
            .with_min_inner_size([1050.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "FFOne UI Editor",
        options,
        Box::new(move |cc| {
            Ok(Box::new(UiEditorApp::new(
                cc,
                layout_path.clone(),
                background_path.clone(),
            )))
        }),
    )
}

mod constants;
mod operations;
mod state;
mod commands;
mod types_ui_editor_app_new;
mod types_ui_editor_app_right_panel;
mod textures;
mod input;
mod output;

use constants::{DEFAULT_LAYOUT, DEFAULT_BACKGROUND, HANDLE_SIZE, HISTORY_LIMIT};
use operations::{arguments, draw_element_visual, draw_grid, group_color};
use state::DragMode;
use commands::AlignAction;
use types_ui_editor_app_new::{ActiveDrag, UiEditorApp};
use textures::{load_texture, rgba};
use input::{load_form_textures, load_element_textures};
use output::install_document_fonts;
