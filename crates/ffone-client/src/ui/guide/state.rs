use super::*;

pub const GUIDE_UI_PARITY_STATUS: &str = "partial";

pub const GUIDE_FIRST_CHANGE_NEXT_MODE: i32 = 10;

#[derive(Component, Debug)]
pub struct GuideUiSelectionWindow;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct GuideUiMentorSelectedEffect {
    pub(super) mentor: GuideMentor,
}

#[derive(Component, Debug)]
pub(super) struct GuideUiSelectionHeading;

#[derive(Component, Debug)]
pub(super) struct GuideUiSelectionIntro;

#[allow(clippy::type_complexity)]
pub(super) fn sync_guide_ui_selection(
    model: Res<GuideUiModel>,
    assets: Res<GuideUiAssets>,
    mut copy: Query<
        (
            &mut LocalizedText,
            Option<&GuideUiSelectionHeading>,
            Option<&GuideUiSelectionIntro>,
            Option<&GuideUiCostText>,
        ),
        (
            Or<(
                With<GuideUiSelectionHeading>,
                With<GuideUiSelectionIntro>,
                With<GuideUiCostText>,
            )>,
            Without<GuideUiButtonLabel>,
        ),
    >,
    mut display_nodes: Query<
        (
            &mut Node,
            Option<&GuideUiCostGroup>,
            Option<&GuideUiMentorCurrentFrame>,
            Option<&GuideUiMentorSelectedEffect>,
        ),
        Or<(
            With<GuideUiCostGroup>,
            With<GuideUiMentorCurrentFrame>,
            With<GuideUiMentorSelectedEffect>,
        )>,
    >,
    mut buttons: Query<(&GuideUiCommandButton, &Children, &mut ImageNode)>,
    mut labels: Query<(&GuideUiTextElement, &mut LocalizedText), With<GuideUiButtonLabel>>,
) {
    for (mut localized, heading_marker, intro_marker, cost_marker) in &mut copy {
        if heading_marker.is_some() {
            set_localized_text(
                &mut localized,
                guide_selection_heading_localized_text(model.purpose),
            );
        } else if intro_marker.is_some() {
            set_localized_text(
                &mut localized,
                guide_selection_intro_localized_text(model.purpose),
            );
        } else if cost_marker.is_some() {
            set_localized_text(
                &mut localized,
                guide_cost_localized_text(model.displayed_price().unwrap_or_default()),
            );
        }
    }

    let selection_visible = model.visible && model.phase == GuideUiPhase::MentorSelection;
    for (mut node, cost_group, current_frame, selected_effect) in &mut display_nodes {
        if cost_group.is_some() {
            node.display =
                display(selection_visible && model.purpose == GuideUiPurpose::ChangeMentor);
        } else if let Some(frame) = current_frame {
            node.display = display(
                selection_visible
                    && model.purpose == GuideUiPurpose::ChangeMentor
                    && model.current == Some(frame.mentor),
            );
        } else if let Some(effect) = selected_effect {
            node.display = display(selection_visible && model.selected == Some(effect.mentor));
        }
    }

    for (button, children, mut image) in &mut buttons {
        if let GuideUiButtonVisual::Toggle(mentor) = button.visual {
            image.image = if model.selected == Some(mentor) {
                assets.toggle_on.clone()
            } else {
                assets.toggle_off.clone()
            };
        }
        if button.command == GuideUiCommand::OpenConfirmation {
            for child in children.iter() {
                if let Ok((element, mut localized)) = labels.get_mut(child)
                    && element.role == GuideUiTextRole::PrimaryButton
                {
                    set_localized_text(
                        &mut localized,
                        guide_primary_button_localized_text(model.purpose),
                    );
                }
            }
        }
    }
}
