use super::*;

pub(super) fn apply_gui_layout_label_node(
    node: &mut Node,
    style: TutorialVoiceSubtitleStyleContract,
    scale: f32,
) {
    node.align_self = AlignSelf::Stretch;
    node.flex_grow = 0.0;
    node.flex_shrink = 0.0;
    node.margin = style.margin.scaled_ui_rect(scale);
    node.padding = style.padding.scaled_ui_rect(scale);
    node.overflow = style.overflow();
}

pub(super) fn apply_gui_text_style(
    font: &mut TextFont,
    line_height: &mut LineHeight,
    color: &mut TextColor,
    style: TutorialVoiceSubtitleStyleContract,
    scale: f32,
) {
    font.font_size = (style.semantic_font_size * scale).into();
    *line_height = LineHeight::Px(style.source_line_spacing * scale);
    color.0 = style.color();
}
