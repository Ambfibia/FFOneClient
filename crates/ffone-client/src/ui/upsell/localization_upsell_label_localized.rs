use super::*;

pub(super) fn upsell_label_localized(label: &'static str) -> LocalizedText {
    match label {
        UPSELL_CONTINUE_PLAYING_LABEL => LocalizedText::new("ui.upsell.continue_playing", label),
        UPSELL_NOT_RIGHT_NOW_LABEL => LocalizedText::new("ui.upsell.not_right_now", label),
        UPSELL_CONTINUE_LABEL => LocalizedText::new("ui.common.continue", label),
        _ => LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", label),
    }
}
