use super::*;

pub(super) fn enchant_bind_optional_localized_text_0104(
    node: &mut Node,
    localized: Option<Mut<LocalizedText>>,
    value: Option<LocalizedText>,
) {
    let Some(mut localized) = localized else {
        return;
    };
    if let Some(value) = value {
        if node.display != Display::Flex {
            node.display = Display::Flex;
        }
        if *localized != value {
            *localized = value;
        }
    } else {
        if node.display != Display::None {
            node.display = Display::None;
        }
    }
}
