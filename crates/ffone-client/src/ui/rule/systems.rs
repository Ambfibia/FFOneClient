use super::*;

pub(super) fn sync_rule_ui_page(
    model: Res<RuleUiModel>,
    assets: Res<RuleUiAssets>,
    mut texts: Query<(&RuleUiTextElement, &mut LocalizedText)>,
    mut illustrations: Query<(&RuleUiIllustration, &mut ImageNode)>,
    mut buttons: Query<(&RuleUiButton, &mut Node)>,
) {
    let Some(page) = model.page() else {
        return;
    };
    for (element, mut localized) in &mut texts {
        if let Some(slot) = element.role.page_string_slot() {
            *localized = rule_page_text_localized(page.id, element.role, page.strings[slot]);
        }
    }
    for (illustration, mut image) in &mut illustrations {
        image.image = assets.page_images[page.id.slot()][illustration.slot].clone();
    }
    for (button, mut node) in &mut buttons {
        match button.kind {
            RuleUiButtonKind::Previous => {
                node.display = if model.visible && page.previous.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            RuleUiButtonKind::Next => {
                node.display = if model.visible && page.next.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            RuleUiButtonKind::Close | RuleUiButtonKind::Back => {}
        }
    }
}
