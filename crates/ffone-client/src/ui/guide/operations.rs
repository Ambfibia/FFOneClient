use super::*;

#[must_use]
pub fn guide_ui_view(model: &GuideUiModel, viewport: Vec2) -> Option<GuideUiView> {
    if !model.visible {
        return None;
    }
    let layout = guide_ui_layout(viewport, model.effective_ui_scale(viewport.y));
    let mentors = GuideMentor::CLEAN_ORDER.map(|mentor| {
        let card = mentor.card_rect();
        let icon_size = mentor.icon_size();
        GuideMentorView {
            mentor,
            card_rect: card,
            // `cnGuideMode` intentionally starts each icon at
            // `-GuideSelectIcon.width / 2` relative to the card. The native badge
            // paints separately from the clipped text group so it stays whole.
            icon_rect: GuideUiRect::new(
                card.x - icon_size.x * 0.5,
                card.y,
                icon_size.x,
                icon_size.y,
            ),
            toggle_rect: GUIDE_TOGGLE_RECT.translated(card.x, card.y),
            portrait_rect: mentor.portrait_rect(),
            selected_effect_rect: mentor.selected_effect_rect(),
            current_frame_rect: mentor.current_frame_rect(),
            selected: model.selected == Some(mentor),
            current: model.current == Some(mentor) && model.purpose == GuideUiPurpose::ChangeMentor,
            portrait_path: mentor.portrait_path(),
            icon_path: mentor.icon_path(),
            toggle_path: if model.selected == Some(mentor) {
                GUIDE_TOGGLE_ON_PATH
            } else {
                GUIDE_TOGGLE_OFF_PATH
            },
            name: mentor.name(),
            description: mentor.description(),
        }
    });
    let confirmation = guide_confirmation_view(model);
    let selection = model.phase == GuideUiPhase::MentorSelection;
    Some(GuideUiView {
        layout,
        purpose: model.purpose,
        phase: model.phase,
        backdrop_path: if model.phase == GuideUiPhase::WarpWarning {
            GUIDE_SELECT_BACKGROUND_PATH
        } else {
            GUIDE_CHANGE_BACKGROUND_PATH
        },
        heading: selection.then_some(match model.purpose {
            GuideUiPurpose::InitialSelection => GUIDE_CHOOSE_HEADING,
            GuideUiPurpose::ChangeMentor => GUIDE_CHANGE_HEADING,
        }),
        intro: selection.then_some(match model.purpose {
            GuideUiPurpose::InitialSelection => GUIDE_CHOOSE_INTRO,
            GuideUiPurpose::ChangeMentor => GUIDE_CHANGE_INTRO,
        }),
        primary_label: selection.then_some(match model.purpose {
            GuideUiPurpose::InitialSelection => GUIDE_CHOOSE_BUTTON_LABEL,
            GuideUiPurpose::ChangeMentor => GUIDE_CHANGE_BUTTON_LABEL,
        }),
        displayed_cost: match model.purpose {
            GuideUiPurpose::InitialSelection => None,
            GuideUiPurpose::ChangeMentor => model
                .displayed_price()
                .map(|price| format!("{GUIDE_COST_LABEL} : {price}")),
        },
        mentors,
        confirmation,
    })
}

pub(super) fn guide_confirmation_view(model: &GuideUiModel) -> Option<GuideConfirmationView> {
    if !model.confirmation_open {
        return None;
    }
    let mentor = model.selected?;
    let (title, body) = match model.purpose {
        GuideUiPurpose::InitialSelection => (
            GUIDE_CONFIRM_TITLE,
            format!(
                "Are you sure that you want to choose {} as your guide?",
                mentor.name()
            ),
        ),
        GuideUiPurpose::ChangeMentor => (
            GUIDE_CHANGE_CONFIRM_TITLE,
            format!(
                "Are you sure you want to change your guide to {}? Remember, once you change \
guides, you will no longer be able to equip your current guide items, and any guide items that \
your currently have equipped will no longer be functional.",
                mentor.name()
            ),
        ),
    };
    Some(GuideConfirmationView {
        mentor,
        art_path: mentor.confirm_path(),
        art_rect: mentor.confirm_art_rect(),
        title,
        body,
    })
}

pub(super) fn guide_passthrough_text(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
}

pub(super) fn absolute_node(rect: GuideUiRect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.x),
        top: px(rect.y),
        width: px(rect.width),
        height: px(rect.height),
        ..default()
    }
}
