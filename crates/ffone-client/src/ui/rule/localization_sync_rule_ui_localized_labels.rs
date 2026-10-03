use super::*;

pub(super) fn rule_page_text_localized(
    page: RulePageId,
    role: RuleUiTextRole,
    fallback: impl Into<String>,
) -> LocalizedText {
    let slot = role
        .page_string_slot()
        .expect("page text roles must map to a clean RulesTable slot");
    let key = match page {
        RulePageId::Vehicle => RULE_UI_VEHICLE_TEXT_KEYS[slot],
        RulePageId::Combining => RULE_UI_COMBINING_TEXT_KEYS[slot],
    };
    LocalizedText::new(key, fallback)
}

pub(super) fn rule_button_localized(role: RuleUiTextRole, fallback: impl Into<String>) -> LocalizedText {
    let key = match role {
        RuleUiTextRole::BackButton => "ui.common.back",
        RuleUiTextRole::PreviousButton => "ui.common.previous",
        RuleUiTextRole::NextButton => "ui.common.next",
        _ => "ui.content.passthrough",
    };
    if key == "ui.content.passthrough" {
        LocalizedText::new(key, "{text}").with_arg("text", fallback)
    } else {
        LocalizedText::new(key, fallback)
    }
}

pub(super) fn sync_rule_ui_localized_labels(
    labels: Res<RuleUiLabels>,
    mut texts: Query<(&RuleUiTextElement, &mut LocalizedText)>,
) {
    if !labels.is_changed() {
        return;
    }
    for (element, mut localized) in &mut texts {
        let label = match element.role {
            RuleUiTextRole::BackButton => Some(&labels.back),
            RuleUiTextRole::PreviousButton => Some(&labels.previous),
            RuleUiTextRole::NextButton => Some(&labels.next),
            _ => None,
        };
        if let Some(label) = label {
            *localized = rule_button_localized(element.role, label);
        }
    }
}
