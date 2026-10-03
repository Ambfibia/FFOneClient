use super::*;

#[derive(Bundle)]
pub(super) struct EditorTextBundle {
    pub(super) text: Text,
    pub(super) font: TextFont,
    pub(super) color: TextColor,
    pub(super) layout: TextLayout,
    pub(super) localized: LocalizedText,
}
