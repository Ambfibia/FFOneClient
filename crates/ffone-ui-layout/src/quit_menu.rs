//! Editable native quit dialog. Event identities refer to existing game handlers.
use serde::{Deserialize, Serialize};

pub const QUIT_MENU_DOCUMENT_PATH: &str = "ui/en/gameplay/quit-menu/menu.ffquit.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuitMenuDocument {
    pub schema: String,
    pub dialog_size: [f32; 2],
    pub reference_height: f32,
    pub scale_nudge: f32,
    pub backdrop: String,
    pub dialog: String,
    pub font: String,
    pub font_size: f32,
    pub line_height: f32,
    pub styles: [QuitMenuStyle; 2],
    pub buttons: [QuitMenuControl; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuitMenuStyle {
    /// GUIStyle backgrounds cover the control Rect independently of text padding.
    #[serde(default)]
    pub background_box: QuitMenuBackgroundBox,
    pub normal: String,
    pub hover: String,
    /// Missing in older native documents: retain their accepted pressed image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,
    /// left, right, top, bottom
    pub border: [f32; 4],
    pub padding: [f32; 4],
    pub normal_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub active_color: [f32; 4],
    pub word_wrap: bool,
    pub clips_text: bool,
    pub content_offset: [f32; 2],
    /// Measured replacement-font translation, separate from Unity contentOffset.
    #[serde(default, skip_serializing_if = "is_zero_offset")]
    pub font_compensation: [f32; 2],
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuitMenuBackgroundBox {
    #[default]
    BorderBox,
}

fn is_zero_offset(offset: &[f32; 2]) -> bool {
    *offset == [0.0; 2]
}

impl QuitMenuStyle {
    pub fn text_offset(&self) -> [f32; 2] {
        std::array::from_fn(|i| self.content_offset[i] + self.font_compensation[i])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuitMenuAction {
    ChangeCharacter,
    QuitGame,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuitMenuControl {
    pub action: QuitMenuAction,
    pub localization_key: String,
    pub fallback: String,
    /// x, y, width, height relative to the centered dialog.
    pub rect: [f32; 4],
    pub style: usize,
}

impl QuitMenuDocument {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        let document: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        document.validate()?;
        Ok(document)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "ffone.quit-menu.v1" {
            return Err("unsupported quit menu schema".into());
        }
        for value in self.dialog_size.into_iter().chain([
            self.reference_height,
            self.scale_nudge,
            self.font_size,
            self.line_height,
        ]) {
            if !value.is_finite() || value <= 0.0 {
                return Err("invalid quit menu geometry/metrics".into());
            }
        }
        for path in [&self.backdrop, &self.dialog, &self.font]
            .into_iter()
            .chain(self.styles.iter().flat_map(|s| [&s.normal, &s.hover]))
            .chain(self.styles.iter().filter_map(|s| s.active.as_ref()))
        {
            if path.is_empty()
                || path.contains(['\\', ':'])
                || path
                    .split('/')
                    .any(|p| p.is_empty() || p == "." || p == "..")
            {
                return Err(format!("unsafe native UI dependency {path}"));
            }
        }
        let mut actions = Vec::new();
        for control in &self.buttons {
            if control.style >= self.styles.len()
                || control.localization_key.is_empty()
                || control.fallback.is_empty()
                || !control.rect.iter().all(|x| x.is_finite())
                || control.rect[2] <= 0.0
                || control.rect[3] <= 0.0
                || actions.contains(&control.action)
            {
                return Err("invalid or duplicate quit menu control".into());
            }
            actions.push(control.action);
        }
        for style in &self.styles {
            if !style
                .border
                .iter()
                .chain(&style.padding)
                .all(|x| x.is_finite() && *x >= 0.0)
                || !style.content_offset.iter().all(|x| x.is_finite())
                || !style.font_compensation.iter().all(|x| x.is_finite())
                || !style.text_offset().iter().all(|x| x.is_finite())
                || !style
                    .normal_color
                    .iter()
                    .chain(&style.hover_color)
                    .chain(&style.active_color)
                    .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
            {
                return Err("invalid quit menu style".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
