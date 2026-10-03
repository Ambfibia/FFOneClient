use super::*;

pub(super) const fn spec(cue: &'static str, line: i32, max_seconds: f32) -> TutorialVoiceSubtitleSpec {
    TutorialVoiceSubtitleSpec {
        cue,
        event: TUTORIAL_VOICE_SUBTITLE_EVENT,
        line,
        max_seconds,
        english_literal_override: None,
    }
}

pub(super) const fn override_spec(
    cue: &'static str,
    line: i32,
    max_seconds: f32,
    literal: &'static str,
) -> TutorialVoiceSubtitleSpec {
    TutorialVoiceSubtitleSpec {
        cue,
        event: TUTORIAL_VOICE_SUBTITLE_EVENT,
        line,
        max_seconds,
        english_literal_override: Some(literal),
    }
}

pub(super) fn gui_layout_label_node(style: TutorialVoiceSubtitleStyleContract, scale: f32) -> Node {
    Node {
        align_self: AlignSelf::Stretch,
        flex_shrink: 0.0,
        margin: style.margin.scaled_ui_rect(scale),
        padding: style.padding.scaled_ui_rect(scale),
        overflow: style.overflow(),
        ..default()
    }
}

pub(super) fn gui_text_font(
    font: Handle<Font>,
    style: TutorialVoiceSubtitleStyleContract,
    scale: f32,
) -> (TextFont, LineHeight) {
    (
        TextFont {
            font: (font).into(),
            font_size: (style.semantic_font_size * scale).into(),
            ..default()
        },
        LineHeight::Px(style.source_line_spacing * scale),
    )
}

pub(super) fn gui_text_layout(style: TutorialVoiceSubtitleStyleContract) -> TextLayout {
    TextLayout::new(
        Justify::Center,
        if style.word_wrap {
            LineBreak::WordBoundary
        } else {
            LineBreak::NoWrap
        },
    )
}
