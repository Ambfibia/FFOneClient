use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};
use thiserror::Error;

pub const UI_LAYOUT_SCHEMA_V1: &str = "ffone.ui-layout.v1";

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiLayoutRect {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn is_finite_positive(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutSpace {
    pub id: String,
    pub label: String,
    pub origin: [f32; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutForm {
    pub id: String,
    pub label: String,
    pub canvas_size: [f32; 2],
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub background_rect: Option<UiLayoutRect>,
    #[serde(default)]
    pub background_source_rect: Option<UiLayoutRect>,
    #[serde(default)]
    pub space_origins: BTreeMap<String, [f32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutElement {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub form: String,
    pub group: String,
    pub space: String,
    pub rect: UiLayoutRect,
    pub source_rect: UiLayoutRect,
    #[serde(default = "default_true")]
    pub override_enabled: bool,
    #[serde(default)]
    pub editor_hidden: bool,
    #[serde(default)]
    pub locked: bool,
    /// Editor-only visual preview. The game may use the element solely as a
    /// geometry override, while the standalone editor uses this description
    /// to draw the same native image/text layer instead of a screenshot crop.
    #[serde(default)]
    pub visual: Option<UiLayoutVisual>,
    /// Paint order inside a form. Elements with the same value retain their
    /// document order.
    #[serde(default)]
    pub z_index: i32,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutVisual {
    #[serde(default)]
    pub image: Option<UiLayoutImage>,
    #[serde(default)]
    pub text: Option<UiLayoutText>,
    #[serde(default)]
    pub fill: Option<[u8; 4]>,
    /// Draws an explicit checkerboard for runtime-provided images (item icons,
    /// portraits, render textures) without pretending that a static asset is
    /// their authoritative source.
    #[serde(default)]
    pub dynamic_placeholder: bool,
}

impl UiLayoutVisual {
    #[must_use]
    pub fn image(path: impl Into<String>) -> Self {
        Self {
            image: Some(UiLayoutImage {
                path: path.into(),
                ..UiLayoutImage::default()
            }),
            text: None,
            fill: None,
            dynamic_placeholder: false,
        }
    }

    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self {
            image: None,
            text: Some(UiLayoutText {
                value: value.into(),
                ..UiLayoutText::default()
            }),
            fill: None,
            dynamic_placeholder: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutImage {
    pub path: String,
    #[serde(default)]
    pub source_rect: Option<UiLayoutRect>,
    #[serde(default)]
    pub mode: UiLayoutImageMode,
    /// Nine-slice border in left/right/top/bottom source pixels.
    #[serde(default)]
    pub border: [f32; 4],
    #[serde(default = "white_rgba")]
    pub tint: [u8; 4],
}

impl Default for UiLayoutImage {
    fn default() -> Self {
        Self {
            path: String::new(),
            source_rect: None,
            mode: UiLayoutImageMode::Stretch,
            border: [0.0; 4],
            tint: white_rgba(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UiLayoutImageMode {
    #[default]
    Stretch,
    NineSlice,
    Contain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutText {
    pub value: String,
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "white_rgba")]
    pub color: [u8; 4],
    #[serde(default)]
    pub horizontal: UiLayoutHorizontalAlign,
    #[serde(default)]
    pub vertical: UiLayoutVerticalAlign,
    #[serde(default)]
    pub wrap: bool,
}

impl Default for UiLayoutText {
    fn default() -> Self {
        Self {
            value: String::new(),
            font: None,
            font_size: default_font_size(),
            color: white_rgba(),
            horizontal: UiLayoutHorizontalAlign::Left,
            vertical: UiLayoutVerticalAlign::Top,
            wrap: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UiLayoutHorizontalAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UiLayoutVerticalAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

const fn default_font_size() -> f32 {
    12.0
}

const fn white_rgba() -> [u8; 4] {
    [255, 255, 255, 255]
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutDocument {
    pub schema: String,
    pub name: String,
    pub reference_resolution: [f32; 2],
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub forms: Vec<UiLayoutForm>,
    pub spaces: Vec<UiLayoutSpace>,
    pub elements: Vec<UiLayoutElement>,
}

impl UiLayoutDocument {
    pub fn from_json(bytes: &[u8]) -> Result<Self, UiLayoutError> {
        let document: Self = serde_json::from_slice(bytes)?;
        document.validate()?;
        Ok(document)
    }

    pub fn load(path: &Path) -> Result<Self, UiLayoutError> {
        Self::from_json(&std::fs::read(path)?)
    }

    pub fn to_pretty_json(&self) -> Result<Vec<u8>, UiLayoutError> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn validate(&self) -> Result<(), UiLayoutError> {
        if self.schema != UI_LAYOUT_SCHEMA_V1 {
            return Err(UiLayoutError::Schema(self.schema.clone()));
        }
        if !self.reference_resolution[0].is_finite()
            || !self.reference_resolution[1].is_finite()
            || self.reference_resolution[0] <= 0.0
            || self.reference_resolution[1] <= 0.0
        {
            return Err(UiLayoutError::ReferenceResolution);
        }
        let mut spaces = HashSet::new();
        for space in &self.spaces {
            if space.id.trim().is_empty() || !spaces.insert(space.id.as_str()) {
                return Err(UiLayoutError::DuplicateSpace(space.id.clone()));
            }
            if !space.origin.iter().all(|value| value.is_finite()) {
                return Err(UiLayoutError::InvalidSpaceOrigin(space.id.clone()));
            }
        }
        let mut forms = HashSet::new();
        for form in &self.forms {
            if form.id.trim().is_empty() || !forms.insert(form.id.as_str()) {
                return Err(UiLayoutError::DuplicateForm(form.id.clone()));
            }
            if !form
                .canvas_size
                .iter()
                .all(|value| value.is_finite() && *value > 0.0)
            {
                return Err(UiLayoutError::InvalidFormCanvas(form.id.clone()));
            }
            if form
                .background_rect
                .is_some_and(|rect| !rect.is_finite_positive())
                || form
                    .background_source_rect
                    .is_some_and(|rect| !rect.is_finite_positive())
            {
                return Err(UiLayoutError::InvalidFormBackground(form.id.clone()));
            }
            for (space, origin) in &form.space_origins {
                if !spaces.contains(space.as_str()) {
                    return Err(UiLayoutError::MissingFormSpace {
                        form: form.id.clone(),
                        space: space.clone(),
                    });
                }
                if !origin.iter().all(|value| value.is_finite()) {
                    return Err(UiLayoutError::InvalidFormSpaceOrigin {
                        form: form.id.clone(),
                        space: space.clone(),
                    });
                }
            }
        }
        let mut elements = HashSet::new();
        for element in &self.elements {
            if element.id.trim().is_empty() || !elements.insert(element.id.as_str()) {
                return Err(UiLayoutError::DuplicateElement(element.id.clone()));
            }
            if !spaces.contains(element.space.as_str()) {
                return Err(UiLayoutError::MissingSpace {
                    element: element.id.clone(),
                    space: element.space.clone(),
                });
            }
            if !element.form.is_empty() && !forms.contains(element.form.as_str()) {
                return Err(UiLayoutError::MissingElementForm {
                    element: element.id.clone(),
                    form: element.form.clone(),
                });
            }
            if !element.rect.is_finite_positive() || !element.source_rect.is_finite_positive() {
                return Err(UiLayoutError::InvalidRect(element.id.clone()));
            }
            if let Some(visual) = &element.visual {
                if let Some(image) = &visual.image {
                    if image.path.trim().is_empty()
                        || image
                            .source_rect
                            .is_some_and(|rect| !rect.is_finite_positive())
                        || !image
                            .border
                            .iter()
                            .all(|value| value.is_finite() && *value >= 0.0)
                    {
                        return Err(UiLayoutError::InvalidVisual(element.id.clone()));
                    }
                }
                if let Some(text) = &visual.text
                    && (!text.font_size.is_finite() || text.font_size <= 0.0)
                {
                    return Err(UiLayoutError::InvalidVisual(element.id.clone()));
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn rect(&self, id: &str) -> Option<UiLayoutRect> {
        self.elements
            .iter()
            .find(|element| element.id == id && element.override_enabled)
            .map(|element| element.rect)
    }

    #[must_use]
    pub fn space_origin(&self, id: &str) -> Option<[f32; 2]> {
        self.spaces
            .iter()
            .find(|space| space.id == id)
            .map(|space| space.origin)
    }

    #[must_use]
    pub fn form(&self, id: &str) -> Option<&UiLayoutForm> {
        self.forms.iter().find(|form| form.id == id)
    }
}

#[derive(Debug, Error)]
pub enum UiLayoutError {
    #[error("failed to read UI layout: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid UI layout JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported UI layout schema `{0}`")]
    Schema(String),
    #[error("referenceResolution must contain two finite positive values")]
    ReferenceResolution,
    #[error("duplicate or empty space id `{0}`")]
    DuplicateSpace(String),
    #[error("space `{0}` has a non-finite origin")]
    InvalidSpaceOrigin(String),
    #[error("duplicate or empty form id `{0}`")]
    DuplicateForm(String),
    #[error("form `{0}` has an invalid canvas size")]
    InvalidFormCanvas(String),
    #[error("form `{0}` has an invalid background Rect")]
    InvalidFormBackground(String),
    #[error("form `{form}` refers to missing space `{space}`")]
    MissingFormSpace { form: String, space: String },
    #[error("form `{form}` has a non-finite origin for space `{space}`")]
    InvalidFormSpaceOrigin { form: String, space: String },
    #[error("duplicate or empty element id `{0}`")]
    DuplicateElement(String),
    #[error("element `{element}` refers to missing space `{space}`")]
    MissingSpace { element: String, space: String },
    #[error("element `{element}` refers to missing form `{form}`")]
    MissingElementForm { element: String, form: String },
    #[error("element `{0}` has an invalid rect")]
    InvalidRect(String),
    #[error("element `{0}` has an invalid editor visual")]
    InvalidVisual(String),
}

#[cfg(test)]
mod tests;

pub mod quit_menu;
