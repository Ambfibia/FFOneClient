use super::*;

pub(super) fn valid_level(value: i32) -> Result<u8, UpsellUiError> {
    u8::try_from(value)
        .ok()
        .filter(|level| (1..=4).contains(level))
        .ok_or(UpsellUiError::InvalidLevel(value))
}

pub(super) fn continue_news(
    model: &mut UpsellUiModel,
    outbox: &mut UpsellUiOutbox,
    audio: &mut UpsellUiAudioOutbox,
) -> bool {
    let page_count = model.news_page_paths.len();
    if page_count <= 1 {
        model.close();
        outbox.push(UpsellUiAction::Exit);
        return true;
    }
    if model.news_page >= page_count {
        return false;
    }

    // The source calls ButtonSound before advancing and does so even when the
    // modulo wrap immediately exits.
    audio.push(UpsellUiAudioCue::ButtonSound);
    let next_page = (model.news_page + 1) % page_count;
    if next_page == 0 {
        model.close();
        outbox.push(UpsellUiAction::Exit);
    } else {
        model.news_page = next_page;
        model.page_alpha = UPSELL_PAGE_CHANGE_START_ALPHA;
        audio.push(UpsellUiAudioCue::ActionSuccess);
    }
    true
}

pub fn dismiss_upsell_with_escape(model: &mut UpsellUiModel, outbox: &mut UpsellUiOutbox) -> bool {
    if !model.escape_dismiss_enabled() {
        return false;
    }
    model.close();
    outbox.push(UpsellUiAction::Exit);
    true
}
