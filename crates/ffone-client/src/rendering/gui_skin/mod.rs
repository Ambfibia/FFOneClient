//! Compatibility view of the historical Retrobution `GUISkin` snapshot.
//!
//! This predates strict object evidence and remains only for already-migrated UI slices. The current
//! FusionForge converter emits a non-publishable candidate schema which this module rejects.
//! New UI must use source-neutral native style contracts and must not extend this snapshot or use
//! Unity IDs as runtime lookup keys. The runtime never reads a Unity container or editor cache.

use std::{collections::BTreeMap, sync::OnceLock};

use serde::Deserialize;

const RETROBUTION_GUI_SKINS: &str =
    include_str!("../../../../../assets/game/ui/en/gameplay/skins/retrobution-20260613.json");

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuiInsets {
    pub left: i64,
    pub right: i64,
    pub top: i64,
    pub bottom: i64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuiObjectPointer {
    pub file_id: i64,
    pub path_id: i64,
    #[serde(default)]
    pub asset_name: Option<String>,
    #[serde(default)]
    pub asset_type: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
pub struct GuiVector2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
pub struct GuiColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuiStyleState {
    pub background: GuiObjectPointer,
    pub text_color: GuiColor,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuiStyle {
    pub name: String,
    pub font: GuiObjectPointer,
    pub states: BTreeMap<String, GuiStyleState>,
    pub border: GuiInsets,
    pub margin: GuiInsets,
    pub padding: GuiInsets,
    pub overflow: GuiInsets,
    pub content_offset: GuiVector2,
    pub image_position: i64,
    pub alignment: i64,
    pub word_wrap: i64,
    pub text_clipping: i64,
    pub fixed_width: f64,
    pub fixed_height: f64,
    pub stretch_width: i64,
    pub stretch_height: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GuiSkin {
    name: String,
    font: GuiObjectPointer,
    built_in_styles: BTreeMap<String, GuiStyle>,
    custom_styles: Vec<GuiStyle>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GuiSkinContract {
    skins: Vec<GuiSkin>,
}

fn parse_gui_skin_contract(input: &str) -> Result<GuiSkinContract, String> {
    let document: serde_json::Value =
        serde_json::from_str(input).map_err(|error| format!("invalid GUI-skin JSON: {error}"))?;
    for marker in [
        "evidenceLevel",
        "publicationAllowed",
        "limitations",
        "unresolvedPointerCount",
        "diagnostics",
    ] {
        if document.get(marker).is_some() {
            return Err(format!(
                "editor-only GUI-skin candidate marker '{marker}' is forbidden in FFOne runtime"
            ));
        }
    }
    let schema = document
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if schema != "ffone.legacy-unity-gui-skins.v1" {
        return Err(format!("unsupported historical GUI-skin schema '{schema}'"));
    }
    let source_build = document
        .get("sourceBuild")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if source_build != "retrobution-20260613" {
        return Err(format!(
            "unsupported historical GUI-skin source '{source_build}'"
        ));
    }
    serde_json::from_value(document).map_err(|error| format!("invalid GUI-skin contract: {error}"))
}

fn contract() -> &'static GuiSkinContract {
    static CONTRACT: OnceLock<GuiSkinContract> = OnceLock::new();
    CONTRACT.get_or_init(|| {
        parse_gui_skin_contract(RETROBUTION_GUI_SKINS)
            .expect("historical Retrobution GUISkin compatibility snapshot must remain valid")
    })
}

/// Returns a style from the historical compatibility snapshot. Built-in style names use the
/// normalized names from the old converter (`box`, `button`, `textArea`, ...);
/// custom names follow the serialized skin and are matched case-insensitively,
/// mirroring legacy `GUISkin.GetStyle`.
pub fn gui_style(skin_name: &str, style_name: &str) -> Option<&'static GuiStyle> {
    let skin = gui_skin(skin_name)?;

    gui_style_from_skin(skin, style_name)
}

/// Returns the style's explicit font or the containing `GUISkin.m_Font`
/// when Unity would inherit it.
pub fn gui_effective_font(skin_name: &str, style_name: &str) -> Option<&'static GuiObjectPointer> {
    let skin = gui_skin(skin_name)?;
    let style = gui_style_from_skin(skin, style_name)?;
    Some(if style.font.path_id != 0 {
        &style.font
    } else {
        &skin.font
    })
}

fn gui_skin(skin_name: &str) -> Option<&'static GuiSkin> {
    contract().skins.iter().find(|skin| skin.name == skin_name)
}

fn gui_style_from_skin<'a>(skin: &'a GuiSkin, style_name: &str) -> Option<&'a GuiStyle> {
    skin.built_in_styles.get(style_name).or_else(|| {
        skin.custom_styles
            .iter()
            .find(|style| style.name.eq_ignore_ascii_case(style_name))
    })
}

#[cfg(test)]
mod tests;
