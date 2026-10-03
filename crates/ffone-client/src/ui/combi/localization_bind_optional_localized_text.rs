use super::*;

#[must_use]
pub fn localized_combi_equipped_item_message() -> LocalizedText {
    LocalizedText::new("ui.combi.modal.equipped_item", COMBI_MESSAGE_260)
}

pub(super) fn bind_localized_text(localized: Option<Mut<LocalizedText>>, value: LocalizedText) {
    let Some(mut localized) = localized else {
        return;
    };
    *localized = value;
}

pub(super) fn bind_optional_localized_text(
    node: &mut Node,
    localized: Option<Mut<LocalizedText>>,
    value: Option<LocalizedText>,
) {
    let Some(mut localized) = localized else {
        return;
    };
    match value {
        Some(value) => {
            node.display = Display::Flex;
            *localized = value;
        }
        None => {
            node.display = Display::None;
            *localized = combi_passthrough_text("");
        }
    }
}
