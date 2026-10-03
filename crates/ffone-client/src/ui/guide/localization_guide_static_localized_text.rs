use super::*;

pub const GUIDE_MENTOR_NAME_LOCALIZATION_KEYS: [&str; 4] = [
    "ui.guide.mentor.ben_tennyson",
    "ui.guide.mentor.dexter",
    "ui.guide.mentor.mojo_jojo",
    "ui.guide.mentor.edd",
];

pub(super) fn guide_mentor_name_localized_text(mentor: GuideMentor) -> LocalizedText {
    LocalizedText::new(
        GUIDE_MENTOR_NAME_LOCALIZATION_KEYS[mentor.slot()],
        mentor.name(),
    )
}

pub(super) fn guide_static_localized_text(
    role: GuideUiTextRole,
    fallback: impl Into<String>,
) -> LocalizedText {
    let fallback = fallback.into();
    match role {
        GuideUiTextRole::Computress => LocalizedText::new("ui.guide.computress", fallback),
        GuideUiTextRole::SelectionHeading => {
            LocalizedText::new("ui.guide.heading.choose", fallback)
        }
        GuideUiTextRole::SelectionIntro => LocalizedText::new("ui.guide.intro.choose", fallback),
        GuideUiTextRole::MentorName(mentor) => guide_mentor_name_localized_text(mentor),
        GuideUiTextRole::MentorDescription(mentor) => LocalizedText::new(
            format!("content.tabledata.guide.guide_string.{}.sz_string", mentor.wire_id() + 5),
            fallback,
        ),
        GuideUiTextRole::CurrentGuide => LocalizedText::new("ui.guide.current", fallback),
        GuideUiTextRole::Cost => guide_cost_localized_text(0),
        GuideUiTextRole::PrimaryButton => LocalizedText::new("ui.guide.action.choose", fallback),
        GuideUiTextRole::WarpTitle => LocalizedText::new("ui.guide.warp.title", fallback),
        GuideUiTextRole::WarpBody => LocalizedText::new("ui.guide.warp.body", fallback),
        GuideUiTextRole::WarpButton => LocalizedText::new("ui.guide.warp.action", fallback),
        GuideUiTextRole::ConfirmationTitle => {
            LocalizedText::new("ui.guide.confirm.initial.title", fallback)
        }
        GuideUiTextRole::ConfirmationBody => guide_confirmation_body_localized_text(
            GuideUiPurpose::InitialSelection,
            GuideMentor::BenTennyson,
        ),
        GuideUiTextRole::CommonCancel => LocalizedText::new("ui.common.cancel", fallback),
        GuideUiTextRole::CommonConfirm => LocalizedText::new("ui.common.confirm", fallback),
    }
}

pub(super) fn guide_selection_heading_localized_text(purpose: GuideUiPurpose) -> LocalizedText {
    match purpose {
        GuideUiPurpose::InitialSelection => {
            LocalizedText::new("ui.guide.heading.choose", GUIDE_CHOOSE_HEADING)
        }
        GuideUiPurpose::ChangeMentor => {
            LocalizedText::new("ui.guide.heading.change", GUIDE_CHANGE_HEADING)
        }
    }
}

pub(super) fn guide_selection_intro_localized_text(purpose: GuideUiPurpose) -> LocalizedText {
    match purpose {
        GuideUiPurpose::InitialSelection => {
            LocalizedText::new("ui.guide.intro.choose", GUIDE_CHOOSE_INTRO)
        }
        GuideUiPurpose::ChangeMentor => {
            LocalizedText::new("ui.guide.intro.change", GUIDE_CHANGE_INTRO)
        }
    }
}

pub(super) fn guide_primary_button_localized_text(purpose: GuideUiPurpose) -> LocalizedText {
    match purpose {
        GuideUiPurpose::InitialSelection => {
            LocalizedText::new("ui.guide.action.choose", GUIDE_CHOOSE_BUTTON_LABEL)
        }
        GuideUiPurpose::ChangeMentor => {
            LocalizedText::new("ui.guide.action.change", GUIDE_CHANGE_BUTTON_LABEL)
        }
    }
}

pub(super) fn guide_cost_localized_text(price: u32) -> LocalizedText {
    LocalizedText::new("ui.guide.cost", "cost : {price}").with_arg("price", price.to_string())
}

pub(super) fn guide_confirmation_title_localized_text(purpose: GuideUiPurpose) -> LocalizedText {
    match purpose {
        GuideUiPurpose::InitialSelection => {
            LocalizedText::new("ui.guide.confirm.initial.title", GUIDE_CONFIRM_TITLE)
        }
        GuideUiPurpose::ChangeMentor => {
            LocalizedText::new("ui.guide.confirm.change.title", GUIDE_CHANGE_CONFIRM_TITLE)
        }
    }
}

pub(super) fn guide_confirmation_body_localized_text(
    purpose: GuideUiPurpose,
    mentor: GuideMentor,
) -> LocalizedText {
    guide_confirmation_body_localized_text_with_name(purpose, mentor.name())
}

pub(super) fn guide_confirmation_body_localized_text_with_name(
    purpose: GuideUiPurpose,
    mentor_name: impl Into<String>,
) -> LocalizedText {
    let mentor_name = mentor_name.into();
    match purpose {
        GuideUiPurpose::InitialSelection => LocalizedText::new(
            "ui.guide.confirm.initial.body",
            "Are you sure that you want to choose {mentor} as your guide?",
        )
        .with_arg("mentor", mentor_name),
        GuideUiPurpose::ChangeMentor => LocalizedText::new(
            "ui.guide.confirm.change.body",
            "Are you sure you want to change your guide to {mentor}? Remember, once you change \
guides, you will no longer be able to equip your current guide items, and any guide items that \
your currently have equipped will no longer be functional.",
        )
        .with_arg("mentor", mentor_name),
    }
}

pub(super) fn guide_localized_mentor_name(
    mentor: GuideMentor,
    localization: Option<&Localization>,
    language: Option<&Language>,
) -> String {
    let localized = guide_mentor_name_localized_text(mentor);
    localization.zip(language).map_or_else(
        || mentor.name().to_owned(),
        |(localization, language)| localization.text(language, &localized),
    )
}

pub(super) fn set_localized_text(current: &mut LocalizedText, next: LocalizedText) {
    if *current != next {
        *current = next;
    }
}
