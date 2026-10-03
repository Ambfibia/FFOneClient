use super::*;

/// Shrinks a bounded UI text block only when its translated copy does not fit.
///
/// The source rectangle remains authoritative: this component changes the
/// font metrics, never the control geometry. Text is first measured at the
/// source font size and may shrink down to `min_font_size` on following
/// post-layout passes.
#[derive(Clone, Debug, Component)]
pub struct UiTextAutoFit {
    pub(super) max_width: f32,
    pub(super) max_height: f32,
    pub(super) max_font_size: f32,
    pub(super) min_font_size: f32,
    pub(super) max_line_height: LineHeight,
    pub(super) measured_text: String,
}

/// The fixed content rectangle supplying an automatically admitted label.
#[derive(Component)]
pub(super) struct UiTextFitRegion(pub(super) Entity);

impl UiTextAutoFit {
    #[must_use]
    pub fn new(max_width: f32, max_height: f32, font: &(TextFont, LineHeight)) -> Self {
        let max_font_size = font.0.font_size.eval(Vec2::ZERO, 16.0).max(1.0);
        Self {
            max_width: max_width.max(1.0),
            max_height: max_height.max(1.0),
            max_font_size,
            // The translated label remains readable and cannot collapse into
            // an arbitrarily tiny glyph strip.
            min_font_size: 6.0_f32.min(max_font_size),
            max_line_height: font.1,
            measured_text: String::new(),
        }
    }
}
